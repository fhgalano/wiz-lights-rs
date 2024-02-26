use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use surrealdb::sql::Thing;
/// Placeholder to develop internal storage for bulbs.
/// it might be time to start server_side code?
use crate::bulb::Bulb;
use crate::utils::surrealdb_tools::Ids;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct DbBulb {
    id: Option<Thing>,
    bulb: Bulb,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Group {
    id: Option<Thing>,
    name: String,
}

async fn store_bulb(bulb: Bulb, db: &Surreal<Db>) -> surrealdb::Result<()> {
    let db_bulb = DbBulb {
        id: None,
        bulb: bulb.clone(),
    };
    let _: Vec<DbBulb> = db
        .create("bulb")
        .content(db_bulb)
        .await?;

    Ok(())
    // add bulb to db as a node
}

async fn get_bulbs(db: &Surreal<Db>) -> surrealdb::Result<Vec<Bulb>> {
    let b: Vec<DbBulb> = db.select("bulb").await?;
    dbg!(b.clone());
    let bulbs: Vec<Bulb> = b.into_iter().map(|n| n.bulb).collect();
    Ok(bulbs)
}

async fn get_db_bulbs(db: &Surreal<Db>) -> surrealdb::Result<Vec<DbBulb>> {
    db.select("bulb").await
}

async fn store_group(group_name: String, db: &Surreal<Db>) -> surrealdb::Result<Group> {
    let group = Group {
        id: None,
        name: group_name,
    };

    let stored_group: Vec<Group> = db
        .create("grop:0")
        .content(group)
        .await?;

    return Ok(stored_group[0].clone())
}

async fn link_group(db: &Surreal<Db>, group: Group, members: Vec<DbBulb>) -> surrealdb::Result<()> {
    let query = "
        let $in = type::thing($group, $g_id);
        let $out = type::thing($bulb, $b_id);
        RELATE $in->collects->$out;
    ";
    let group_info = group.id.unwrap();

    for m in members.iter() {
        let m_info = m.clone().id.unwrap();
        let q = db
            .query(query)
            .bind(("group", &group_info.tb))
            .bind(("g_id", &group_info.id))
            .bind(("bulb", &m_info.tb))
            .bind(("b_id", &m_info.id))
            .await?;
        dbg!(q);
    };

    Ok(())
}

async fn collect_group(db: &Surreal<Db>, group: Group) -> surrealdb::Result<()> {
    // let query = "
    //     SELECT ->collects->$bulb FROM $group;
    // ";
    // let query = "
    //     LET $tbl = type::table($gr);
    //     let $tb = type::table(bulb);
    //     SELECT collects<- FROM $tbl;
    // ";
    let query = "SELECT ->collects.out from grop:0; INFO FOR DB; RETURN type::table($table);";
    let group_info = group.clone().id.unwrap();

    let mut q = db.query(query)
        .bind(("table", "grop"))
        .await?;

    // let f: Vec<Bulb> = q.take(1).unwrap();
    // dbg!(f.clone());
    // let i: Vec<DbBulb> = q.take(1).unwrap();
    // dbg!(i.clone());
    dbg!(q);

    return Ok(())



}
//
// fn add_function(taco: SomeFuncStruct) {
//     // add function as a node in the database, but missing some stuff
//     // create links between affected elements
// }
//
// fn register_function(f: SomeFuncStruct) {
//     // create graph node for function
// }

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};
    use super::*;
    use rstest::{rstest, fixture};
    use surrealdb::Surreal;
    use surrealdb::engine::local::{Db, Mem};
    use crate::bulb::tests::test_bulb;


    async fn create_memory_db() -> Surreal<Db> {
        // will need async code to start up the local db
        Surreal::new::<Mem>(()).await.unwrap()
    }

    #[rstest]
    #[tokio::test]
    async fn test_registry(test_bulb: Bulb) {
        let db = create_memory_db().await;

        // selecting a specific namespace / database
        db.use_ns("test").use_db("test").await.unwrap();

        store_bulb(test_bulb.clone(), &db).await.unwrap();
        let stored_bulb = get_bulbs(&db).await.unwrap();

        dbg!(stored_bulb[0].clone());
        dbg!(test_bulb.clone());

        assert_eq!(test_bulb, stored_bulb[0])
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_group() {
        let b1 = Bulb::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 68, 01)), "b1".to_string()
        );
        let b2 = Bulb::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 68, 02)), "b2".to_string()
        );

        let db = create_memory_db().await;

        db.use_ns("test").use_db("test").await.unwrap();

        for b in [b1, b2] {
            store_bulb(b.clone(), &db).await.unwrap();
        }

        let sg = store_group("test_group".to_string(), &db).await.unwrap();
        dbg!(sg.clone());

        link_group(
            &db,
            sg.clone(),
            get_db_bulbs(&db).await.unwrap()
        ).await.unwrap();

        let collected_group = collect_group(&db, sg).await.unwrap();

        assert!(true)

    }

    // #[rstest]
    // #[tokio::test]
    // async fn test_store_function() {
    //
    // }
}
