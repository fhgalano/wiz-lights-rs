mod group;
mod bulb;
mod surreal;

use std::fmt;
use std::error::Error;
use std::fmt::Formatter;
use serde::{Serialize, Deserialize};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::sql::{Id, Thing};
use url::Url;

use crate::bulb::Bulb;
use crate::function::*;
use group::Group;
use surreal::{connect_to_db, GraphStore};
use crate::bulb::response::ErrorResponse;
use crate::function::FunctionError;


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
    async fn new(db: Surreal<Any>) -> Registry {
        db.use_ns("test").use_db("test").await.unwrap(); // todo: proper namespaces
        
        let bulbs = get_bulbs_from_db(&db).await.unwrap();
        let groups = get_groups_from_db(&db).await.unwrap();

        Registry {
            db,
            bulbs,
            groups,
        }
    }
    
    async fn new_from_url(url: Url) -> Registry {
        let db = connect_to_db(url).await;

        Registry::new(db).await
    }
    
    async fn add(&mut self, item: Box<dyn GraphStore>) -> surrealdb::Result<()> {
        match item.store(&self.db).await {
            Ok(_) => {
                let tr = Registry::new(self.db.clone()).await;
                self.bulbs = tr.bulbs;
                self.groups = tr.groups;
            },
            Err(e) => return Err(e),
        };
        
        Ok(())
    }

    fn turn_on_by_id(&self, id: Id) -> Result<bool, FunctionError> {
        for i in self.bulbs.iter().clone() {
            if Id::from(i._id.clone() as i32) == id {
                return i.on().map_err(|e| {
                    FunctionError::new(
                        "On".to_string(),
                        e.to_string(),
                    )
                })
            };
        }

        for i in self.groups.iter() {
            if i._id == id {
                return i.on().map_err(|e| {
                    FunctionError::new(
                        "On".to_string(),
                        e.to_string(),
                    )
                })
            }
        }

        Err(FunctionError::new(
            "turn_on_by_id".to_string(),
            MissingElementError {
                _id: id,
            }.to_string(),
        ))
    }

    fn turn_off_by_id(&self, id: Id) -> Result<bool, FunctionError> {
        for i in self.bulbs.iter().clone() {
            if Id::from(i._id.clone() as i32) == id {
                return i.off().map_err(|e| {
                    FunctionError::new(
                        "Off".to_string(),
                        e.to_string(),
                    )
                })
            };
        }

        for i in self.groups.iter() {
            if i._id == id {
                return i.off().map_err(|e| {
                    FunctionError::new(
                        "Off".to_string(),
                        e.to_string(),
                    )
                })
            }
        }

        Err(FunctionError::new(
            "turn_off_by_id".to_string(),
            MissingElementError {
                _id: id,
            }.to_string(),
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
        .bind(("table", "group")).await?;

    let group_ids: Vec<Thing> = db_groups.take((0, "id"))?;
    let mut groups: Vec<Group> = vec![];
    for i in group_ids.into_iter() {
        groups.push(Group::collect(i.id, db).await?);
    }

    Ok(groups)
}


#[cfg(test)]
pub mod tests {
    // use std::net::{IpAddr, Ipv4Addr};
    use super::*;
    use rstest::{rstest, fixture};
    use surrealdb::Surreal;
    use surrealdb::engine::local::{Db, Mem};
    use surrealdb::engine::any::Any;
    use crate::bulb::tests::test_bulb;
    use crate::registry::group::tests::test_group;

    pub async fn connect_to_memory_db() -> Surreal<Any> {
        let db = surrealdb::engine::any::connect("ws://localhost:8000").await.unwrap();
        db
    }

    pub async fn create_memory_db() -> Surreal<Any> {
        // will need async code to start up the local db
        surrealdb::engine::any::connect("mem://").await.unwrap()
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
    async fn test_turn_on_bulb_by_id(test_bulb: Bulb) {
        let t_id = Id::from(test_bulb._id.clone() as i32);
        let registry = Registry {
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
        let registry = Registry {
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
        let registry = Registry {
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
        let registry = Registry {
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
    async fn test_add_group(test_bulb: Bulb) {
        let mut registry = Registry::new(create_memory_db().await).await;

        let g0 = Group::new(
            Id::from(22),
            "deez".to_string(),
            vec!(Box::new(test_bulb.clone())),
        );
        let g1 = Group::new(
            Id::from(22),
            "deez".to_string(),
            vec!(Box::new(test_bulb)),
        );
        
        registry.add(Box::new(g0)).await.unwrap();
        
        assert_eq!(registry.groups, vec![g1])
    }
}
