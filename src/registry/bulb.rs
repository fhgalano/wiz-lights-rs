use std::any::Any;
use std::future::Future;
use async_trait::async_trait;
use surrealdb::engine::local::Db;
use surrealdb::sql::Id;
use surrealdb::Surreal;
use crate::bulb::Bulb;
use crate::registry::surreal::{GraphStore, GraphLink};


impl Bulb {
    pub async fn get(db: &Surreal<Db>, id: Id) -> surrealdb::Result<Self> {
        let b: Option<Bulb> = db.select(("bulb", id.to_raw().to_owned())).await?;
        Ok(b.unwrap())
    }
}

#[async_trait]
#[typetag::serde]
impl GraphLink for Bulb {
    fn query_id_string(&self) -> String {
        format!("bulb:{id}", id=self._id)
    }
}

#[async_trait]
#[typetag::serde]
impl GraphStore for Bulb {
    async fn store(&self, db: &Surreal<Db>) -> surrealdb::Result<()> {
        let _: Option<Bulb> = db
            .create(("bulb", self._id.to_string()))
            .content(self)
            .await?;

        Ok(())
    }

    fn upcast(&self) -> &dyn GraphLink {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq(&self, etc: &dyn GraphStore) -> bool {
        match etc.as_any().downcast_ref::<Bulb>() {
            Some(other) => self == other,
            None => false
        }
    }
}


#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use rstest::{rstest, fixture};

    use crate::bulb::Bulb;
    use crate::registry::tests::create_memory_db;
    use crate::bulb::tests::test_bulb;

    use super::*;


    #[rstest]
    #[tokio::test]
    async fn test_store_get_bulb(
        #[from(test_bulb)]
        #[with(Ipv4Addr::new(192, 168, 68, 01), 1)]
        b1: Bulb,
    ) {
        let db = create_memory_db().await;
        db.use_ns("test").use_db("test").await.unwrap();

        b1.store(&db).await.unwrap();

        let get_b1 = Bulb::get(&db, Id::from(b1._id as i32)).await.unwrap();

        assert_eq!(b1, get_b1)
    }
}
