use std::fmt::Debug;
use std::any::Any;

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use typetag;


#[async_trait]
#[typetag::serde(tag = "type")]
pub trait GraphLink: Sync + Debug {
    async fn link(&self, to_link: &dyn GraphLink, relationship: String, db: &Surreal<Db>) -> surrealdb::Result<()> {
        let query = format!(
            "RELATE {origin}->{relationship}->{destination};",
            origin=self.query_id_string(),
            destination=to_link.query_id_string()
        );

        let _ = db
            .query(query.as_str())
            .await?;

        Ok(())
    }
    // fn retrieve<T: GraphLink>(self, db: &Surreal<Db>) -> surrealdb::Result<T>;
    fn query_id_string(&self) -> String;
}

#[async_trait]
#[typetag::serde(tag = "type")]
pub trait GraphStore: GraphLink + Debug + Any {
    async fn store(&self, db: &Surreal<Db>) -> surrealdb::Result<()>;
    fn upcast(&self) -> &dyn GraphLink;
    fn as_any(&self) -> &dyn Any;
    fn eq(&self, etc: &dyn GraphStore) -> bool;
}

impl PartialEq for dyn GraphStore {
    fn eq(&self, other: &Self) -> bool {
        GraphStore::eq(self, other)
    }
}
