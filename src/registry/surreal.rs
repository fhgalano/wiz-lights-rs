use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use typetag;


#[async_trait]
#[typetag::serde(tag = "type")]
pub trait GraphLink: Sync {
    async fn link(&self, to_link: &dyn GraphLink, relationship: String, db: &Surreal<Db>) -> surrealdb::Result<()> {
        let query = format!(
            "RELATE {origin}->{relationship}->{destination}",
            origin=self.query_id_string(),
            destination=to_link.query_id_string()
        );
        let q = db
            .query(query.as_str())
            .await?;
        dbg!(q);

        Ok(())
    }
    // fn retrieve<T: GraphLink>(self, db: &Surreal<Db>) -> surrealdb::Result<T>;
    fn query_id_string(&self) -> String;
}

pub trait Store: {

}
