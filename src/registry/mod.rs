mod bulb;
mod group;
mod surreal;

use std::error::Error;
use std::fmt;
use std::fmt::Formatter;

use serde::{Deserialize, Serialize};
use surrealdb::engine::any::Any;
use surrealdb::sql::{Id, Thing};
use surrealdb::Surreal;
use url::Url;

use crate::bulb::Bulb;
use crate::function::FunctionError;
use crate::function::*;
use group::Group;
pub use surreal::{connect_to_db, GraphStore};

#[derive(Debug, Clone)]
struct MissingElementError {
    _id: Id,
}

impl fmt::Display for MissingElementError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Id not contained in registry: {}", self._id)
    }
}

impl Error for MissingElementError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Out {
    pub out: Vec<Thing>,
}

#[derive(Debug)]
pub struct Registry {
    db: Surreal<Any>,
    bulbs: Vec<Bulb>,
    groups: Vec<Group>,
}

impl Registry {
    pub async fn new(db: Surreal<Any>) -> Registry {
        db.use_ns("test").use_db("test").await.unwrap(); // todo: proper namespaces

        let bulbs = get_bulbs_from_db(&db).await.unwrap_or(vec![]);
        let groups = get_groups_from_db(&db).await.unwrap_or(vec![]);

        Registry { db, bulbs, groups }
    }

    pub async fn new_from_url(url: Url) -> Registry {
        let db = connect_to_db(url).await;

        Registry::new(db).await
    }

    pub async fn add(&mut self, item: Box<dyn GraphStore>) -> surrealdb::Result<()> {
        match item.store(&self.db).await {
            Ok(_) => {
                let tr = Registry::new(self.db.clone()).await;
                self.bulbs = tr.bulbs;
                self.groups = tr.groups;
            }
            Err(e) => return Err(e),
        };

        Ok(())
    }

    pub fn find_bulb_by_name(&self, name: String) -> Result<Bulb, GeneralError> {
        for i in self.bulbs.iter() {
            if i.name == name {
                return Ok(i.clone());
            }
        }

        Err(GeneralError {
            msg: "Unable to find bulb by name".to_string(),
        })
    }

    pub fn turn_on_by_id(&mut self, id: Id) -> Result<bool, FunctionError> {
        for i in self.bulbs.iter_mut() {
            if Id::from(i._id.clone() as i32) == id {
                return i
                    .on()
                    .map_err(|e| FunctionError::new("On".to_string(), e.to_string()));
            };
        }

        for i in self.groups.iter_mut() {
            if i._id == id {
                return i
                    .on()
                    .map_err(|e| FunctionError::new("On".to_string(), e.to_string()));
            }
        }

        Err(FunctionError::new(
            "turn_on_by_id".to_string(),
            MissingElementError { _id: id }.to_string(),
        ))
    }

    pub fn turn_off_by_id(&mut self, id: Id) -> Result<bool, FunctionError> {
        for i in self.bulbs.iter_mut() {
            if Id::from(i._id.clone() as i32) == id {
                return i
                    .off()
                    .map_err(|e| FunctionError::new("Off".to_string(), e.to_string()));
            };
        }

        for i in self.groups.iter_mut() {
            if i._id == id {
                return i
                    .off()
                    .map_err(|e| FunctionError::new("Off".to_string(), e.to_string()));
            }
        }

        Err(FunctionError::new(
            "turn_off_by_id".to_string(),
            MissingElementError { _id: id }.to_string(),
        ))
    }
    // fn add_bulb() -> surrealdb::Result<()> {};
    // fn add_group() -> surrealdb::Result<()> {};
    // fn detect_unknown_bulbs_on_network() -> Option<Vec<Ipv4Addr>> {};
}

async fn get_bulbs_from_db(db: &Surreal<Any>) -> surrealdb::Result<Vec<Bulb>> {
    let b: Vec<Bulb> = db.select("bulb").await?;
    dbg!(b.clone());
    Ok(b)
}

async fn get_groups_from_db(db: &Surreal<Any>) -> surrealdb::Result<Vec<Group>> {
    let mut db_groups = db
        .query("SELECT id FROM type::table($table)")
        .bind(("table", "group"))
        .await?;

    let group_ids: Vec<Thing> = db_groups.take((0, "id"))?;
    let mut groups: Vec<Group> = vec![];
    for i in group_ids.into_iter() {
        groups.push(Group::collect(i.id, db).await?);
    }

    Ok(groups)
}

#[derive(Debug)]
pub struct GeneralError {
    msg: String,
}

impl fmt::Display for GeneralError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Yo dude, something wrong has happened - {}", self.msg,)
    }
}

impl Error for GeneralError {}

#[cfg(test)]
pub mod tests {
    // use std::net::{IpAddr, Ipv4Addr};
    use super::*;
    use crate::bulb::tests::test_bulb;
    use crate::registry::group::tests::test_group;
    use rstest::rstest;
    use surrealdb::engine::any::Any;
    use surrealdb::Surreal;

    pub async fn connect_to_memory_db() -> Surreal<Any> {
        match surrealdb::engine::any::connect("ws://localhost:8000").await {
            Ok(db) => db,
            Err(e) => panic!("Issue connecting to localhost:8000 - {}", e),
        }
    }

    // TODO: figure out how to compare small sections to what is in git
    pub async fn create_memory_db() -> Surreal<Any> {
        // will need async code to start up the local db
        match surrealdb::engine::any::connect("mem://").await {
            Ok(db) => db,
            Err(e) => panic!("Issue creating memorydb - {}", e),
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_registry() {
        let url = Url::parse("ws://localhost:8000").unwrap();
        let t_registry = Registry::new_from_url(url).await;

        dbg!(t_registry);
    }

    #[rstest]
    #[tokio::test]
    async fn test_find_bulb_by_name(test_bulb: Bulb) {
        let registry = Registry {
            db: create_memory_db().await,
            bulbs: vec![test_bulb.clone()],
            groups: vec![],
        };

        assert_eq!(
            registry
                .find_bulb_by_name("test_bulb_0".to_string())
                .unwrap(),
            test_bulb
        )
    }

    #[rstest]
    #[tokio::test]
    async fn test_turn_on_bulb_by_id(test_bulb: Bulb) {
        let t_id = Id::from(test_bulb._id.clone() as i32);
        let mut registry = Registry {
            db: create_memory_db().await,
            bulbs: vec![test_bulb],
            groups: vec![],
        };

        let res = registry.turn_on_by_id(t_id).unwrap();
        dbg!(&res);
        assert_eq!(res, true);
    }

    #[rstest]
    #[tokio::test]
    async fn test_turn_off_bulb_by_id(test_bulb: Bulb) {
        let t_id = Id::from(test_bulb._id.clone() as i32);
        let mut registry = Registry {
            db: create_memory_db().await,
            bulbs: vec![test_bulb],
            groups: vec![],
        };

        let res = registry.turn_off_by_id(t_id).unwrap();
        dbg!(&res);
        assert_eq!(res, true);
    }

    #[rstest]
    #[tokio::test]
    async fn test_turn_on_group_by_id(test_group: Group) {
        let t_id = test_group._id.clone();
        let mut registry = Registry {
            db: create_memory_db().await,
            bulbs: vec![],
            groups: vec![test_group],
        };

        let res = registry.turn_on_by_id(t_id).unwrap();
        dbg!(&res);
        assert_eq!(res, true);
    }

    #[rstest]
    #[tokio::test]
    async fn test_turn_off_group_by_id(test_group: Group) {
        let t_id = test_group._id.clone();
        let mut registry = Registry {
            db: create_memory_db().await,
            bulbs: vec![],
            groups: vec![test_group],
        };

        let res = registry.turn_off_by_id(t_id).unwrap();
        dbg!(&res);
        assert_eq!(res, true);
    }

    #[rstest]
    #[tokio::test]
    async fn test_add_bulb(test_bulb: Bulb) {
        let mut registry = Registry::new(create_memory_db().await).await;

        registry.add(Box::new(test_bulb.clone())).await.unwrap();

        assert_eq!(registry.bulbs, vec![test_bulb]);
    }

    #[rstest]
    #[tokio::test]
    async fn test_add_duplicate_bulb(test_bulb: Bulb) {
        let mut registry = Registry::new(create_memory_db().await).await;

        registry.add(Box::new(test_bulb.clone())).await.unwrap();
        match registry.add(Box::new(test_bulb.clone())).await {
            Ok(_) => println!("Success"),
            Err(surrealdb::Error::Db(e)) => {
                println!("Surrealdb Error: {}", e)
            }
            Err(e) => {
                panic!("add bulb error: {}", e)
            }
        };

        assert_eq!(registry.bulbs, vec![test_bulb]);
    }

    #[rstest]
    #[tokio::test]
    async fn test_add_group(test_bulb: Bulb) {
        let mut registry = Registry::new(create_memory_db().await).await;

        let g0 = Group::new(
            Id::from(22),
            "deez".to_string(),
            vec![Box::new(test_bulb.clone())],
        );
        let g1 = Group::new(Id::from(22), "deez".to_string(), vec![Box::new(test_bulb)]);

        registry.add(Box::new(g0)).await.unwrap();

        assert_eq!(registry.groups, vec![g1])
    }
}
