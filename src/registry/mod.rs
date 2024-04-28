mod group;
mod bulb;
mod surreal;

use serde::{Serialize, Deserialize};
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use surrealdb::sql::Thing;
use surrealdb::engine::any::Any;

use crate::bulb::Bulb;
use surreal::connect_to_db;


#[derive(Debug, Clone, Serialize, Deserialize)]
struct Out {
    pub out: Vec<Thing>,
}


async fn get_bulbs(db: &Surreal<Db>) -> surrealdb::Result<Vec<Bulb>> {
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
