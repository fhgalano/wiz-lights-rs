mod group;
mod bulb;
mod surreal;

use serde::{Serialize, Deserialize};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

use crate::bulb::Bulb;
use surreal::connect_to_db;


#[derive(Debug, Clone, Serialize, Deserialize)]
struct Out {
    pub out: Vec<Thing>,
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
                return i.on().map_err(|e| {
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


#[cfg(test)]
pub mod tests {
    // use std::net::{IpAddr, Ipv4Addr};
    // use super::*;
    // use rstest::{rstest, fixture};
    use surrealdb::Surreal;
    use surrealdb::engine::local::{Db, Mem};
    use surrealdb::engine::any::Any;
    use crate::bulb::tests::test_bulb;


    pub async fn create_memory_db() -> Surreal<Any> {
        // will need async code to start up the local db
        surrealdb::engine::any::connect("mem://").await.unwrap()
    }
}
