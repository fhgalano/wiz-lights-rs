use std::any::Any;
use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use surrealdb::error::Db as SDb;
use surrealdb::engine::any;
use surrealdb::sql::Id;
use surrealdb::Surreal;

use crate::bulb::Bulb;
use crate::bulb::response::ErrorResponse;
use crate::function::{Off, On};
use crate::registry::Out;
use crate::registry::surreal::{GraphStore, GraphLink};

#[derive(Serialize, Deserialize, Debug)]
pub struct Group {
    pub _id: Id,
    name: String,
    collects: Vec<Box<dyn GraphStore>>,
}

impl Group {
    pub fn new(id: Id, name: String, collects: Vec<Box<dyn GraphStore>>) -> Group {
        return Group {
            _id: id,
            name,
            collects,
        }
    }

    pub fn collect(group_id: Id, db: &Surreal<any::Any>) -> Pin<Box<dyn Future<Output = surrealdb::Result<Group>> + '_>> {
        Box::pin(async move {
            let query = format!(
                "SELECT ->collect.out FROM group:{id};",
                id = group_id.to_raw(),
            );

            let mut q = db.query(query.as_str()).await?;
            dbg!(&q);

            let found_links: Option<Out> = q.take("->collect")?;
            let mut collected: Vec<Box<dyn GraphStore>> = vec![];

            for link in found_links.unwrap().out {
                if link.tb == "bulb" {
                    let linked_bulb = Bulb::get(db, link.id).await?;
                    collected.push(Box::new(linked_bulb));
                } else if link.tb == "group" {
                    let group: Group = Group::collect(link.id, db).await?;
                    collected.push(Box::new(group));
                }
            }

            let mut b = db.query(format!("SELECT * FROM group:{_id}", _id=group_id).as_str()).await?;
            let c: Option<String> = b.take((0, "name"))?;

            dbg!(&b);
            dbg!(&c);
            Ok(Group::new(
                group_id,
                c.unwrap(),
                collected,
            ))
        })
    }

    async fn get(db: &Surreal<any::Any>, id: Id) -> surrealdb::Result<Group> {
        Group::collect(id, db).await
    }
}

impl PartialEq for Group {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name {
            return false
        }
        else if self._id != other._id {
            return false
        }

        for i in self.collects.iter() {
            if !other.collects.contains(i) {
                return false
            }

        }

        return true
    }
}

impl On for Group {
    fn on(&mut self) -> Result<bool, ErrorResponse> {
        for i in self.collects.iter_mut() {
            i.on()?;
        }

        Ok(true)
    }
}

impl Off for Group {
    fn off(&mut self) -> Result<bool, ErrorResponse> {
        for i in self.collects.iter_mut() {
            i.off()?;
        }

        Ok(true)
    }
}

#[async_trait]
#[typetag::serde]
impl GraphLink for Group {
    fn query_id_string(&self) -> String {
        format!("group:{id}", id=self._id.to_raw())
    }
}

#[async_trait]
#[typetag::serde]
impl GraphStore for Group {
    async fn store(&self, db: &Surreal<any::Any>) -> surrealdb::Result<()> {
        let query = format!(
            "CREATE {tb_id} SET name = \"{name}\";",
            tb_id = self.query_id_string(),
            name = self.name.as_str(),
        );

        let _ = db.query(query.as_str()).await?;

        for c in self.collects.iter().clone() {
            let _ = match c.store(db).await {
                Err(e) => match e {
                    surrealdb::Error::Db(SDb::RecordExists { .. }) => {
                        dbg!("Record already exists in db: {:?}", e);
                        Ok(())
                    },
                    _ => Err(e),
                },
                _ => Ok(())
            };

            self.link(c.upcast(), "collect".to_string(), db).await?;
        }

        Ok(())
    }

    fn upcast(&self) -> &dyn GraphLink {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq(&self, etc: &dyn GraphStore) -> bool {
        match etc.as_any().downcast_ref::<Group>() {
            Some(o) => self == o,
            None => false
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::net::Ipv4Addr;

    use rstest::{rstest, fixture};

    use crate::bulb::Bulb;
    use crate::registry::tests::connect_to_memory_db;
    use crate::bulb::tests::test_bulb;

    use super::*;


    #[fixture]
    pub fn test_collect(test_bulb: Bulb) -> Vec<Box<dyn GraphStore>> {
        vec![Box::new(test_bulb)]
    }

    #[fixture]
    pub fn test_group(
        #[default(Id::from(69))]
        id: Id,
        test_collect: Vec<Box<dyn GraphStore>>,
    ) -> Group {
        Group::new(
            id.clone(),
            format!("test_group_{}", &id),
            test_collect,
        )
    }

    #[rstest]
    #[tokio::test]
    async fn test_store_get_group(
        #[from(test_bulb)]
        #[with(Ipv4Addr::new(192, 168, 68, 01), 1)]
        b1: Bulb,
        #[from(test_bulb)]
        #[with(Ipv4Addr::new(192, 168, 68, 01), 2)]
        b2: Bulb,
    ) {
        let db = connect_to_memory_db().await;

        db.use_ns("test").use_db("test").await.unwrap();

        let test_group = Group::new(
            Id::from(69),
            "test_bulb".to_string(),
            vec!(Box::new(b1), Box::new(b2))
        );

        test_group.store(&db).await.unwrap();

        let collected_group = Group::collect(Id::from(69), &db).await.unwrap();

        // collected_group.eq(&test_group);
        assert_eq!(test_group, collected_group)
        // dbg!(test_group);
        // dbg!(collected_group);
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_nested_group(
        #[from(test_bulb)]
        #[with(Ipv4Addr::new(192, 168, 68, 01), 1)]
        b1: Bulb,
        #[from(test_bulb)]
        #[with(Ipv4Addr::new(192, 168, 68, 01), 2)]
        b2: Bulb,
        #[from(test_bulb)]
        #[with(Ipv4Addr::new(192, 168, 68, 01), 3)]
        b3: Bulb,
        #[from(test_bulb)]
        #[with(Ipv4Addr::new(192, 168, 68, 01), 4)]
        b4: Bulb,
    ) {
        let db = connect_to_memory_db().await;

        db.use_ns("test").use_db("test").await.unwrap();

        let nested_group = Group::new(
            Id::from(420),
            "nested_group".to_string(),
            vec!(Box::new(b3), Box::new(b4))
        );

        nested_group.store(&db).await.unwrap();

        let test_group = Group::new(
            Id::from(69),
            "test_bulb".to_string(),
            vec!(Box::new(b1), Box::new(b2), Box::new(nested_group))
        );

        test_group.store(&db).await.unwrap();

        let collected_group = Group::get(&db, Id::from(69)).await.unwrap();

        assert_eq!(test_group, collected_group)
    }

    #[rstest] fn test_group_off(test_bulb: Bulb) {
        let mut g = Group::new(
            Id::from(22),
            "deez".to_string(),
            vec!(Box::new(test_bulb)),
        );

        assert!(g.off().unwrap());
    }

    #[rstest] fn test_group_on(test_bulb: Bulb) {
        let mut g = Group::new(
            Id::from(22),
            "deez".to_string(),
            vec!(Box::new(test_bulb)),
        );

        assert!(g.on().unwrap());
    }
    
    #[rstest]
    fn test_deserialize_group(test_group: Group) {
        println!("{}", serde_json::to_string(&test_group).unwrap());
    }

    #[rstest]
    fn test_serialize_bulb(test_group: Group) {
        let ser_group = serde_json::to_string(&test_group).unwrap();
        dbg!(serde_json::from_str::<Group>(ser_group.as_str()).unwrap());
    }

}
