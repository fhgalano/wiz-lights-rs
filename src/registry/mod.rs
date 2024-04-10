mod group;
mod bulb;
mod surreal;

use serde::{Serialize, Deserialize};
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use surrealdb::sql::Thing;

use crate::bulb::Bulb;


#[derive(Debug, Clone, Serialize, Deserialize)]
struct Out {
    pub out: Vec<Thing>,
}


async fn get_bulbs(db: &Surreal<Db>) -> surrealdb::Result<Vec<Bulb>> {
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
    // use crate::bulb::tests::test_bulb;


    pub async fn create_memory_db() -> Surreal<Db> {
        // will need async code to start up the local db
        Surreal::new::<Mem>(()).await.unwrap()
    }
}
