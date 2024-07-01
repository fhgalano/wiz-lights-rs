use std::any::Any;
use std::fmt::Debug;

use crate::function::{Off, On};
use async_trait::async_trait;
use surrealdb::engine::any;
use surrealdb::Surreal;
use typetag;
use url::Url;

pub async fn connect_to_db(url: Url) -> Surreal<any::Any> {
    any::connect(url.as_str()).await.unwrap()
}

#[async_trait]
#[typetag::serde(tag = "type")]
pub trait GraphLink: Sync + Debug + Off + On {
    async fn link(
        &self,
        to_link: &dyn GraphLink,
        relationship: String,
        db: &Surreal<any::Any>,
    ) -> surrealdb::Result<()> {
        let query = format!(
            "RELATE {origin}->{relationship}->{destination};",
            origin = self.query_id_string(),
            destination = to_link.query_id_string()
        );

        let _ = db.query(query.as_str()).await?;

        Ok(())
    }
    fn query_id_string(&self) -> String;
}

#[async_trait]
#[typetag::serde(tag = "type")]
pub trait GraphStore: GraphLink + Debug + Any + Send {
    async fn store(&self, db: &Surreal<any::Any>) -> surrealdb::Result<()>;
    fn upcast(&self) -> &dyn GraphLink;
    fn as_any(&self) -> &dyn Any;
    fn eq(&self, etc: &dyn GraphStore) -> bool;
}

impl PartialEq for dyn GraphStore {
    fn eq(&self, other: &Self) -> bool {
        GraphStore::eq(self, other)
    }
}
