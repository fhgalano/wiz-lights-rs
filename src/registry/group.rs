use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Id;

use crate::bulb::Bulb;
use crate::utils::surreal::GraphLink;

#[derive(Serialize, Deserialize)]
struct Group {
    id: Id,
    name: String,
    bulbs: Vec<Bulb>,
    collects: Vec<Box<dyn GraphLink>>,
}

#[async_trait]
#[typetag::serde]
impl GraphLink for Group {
    fn query_id_string(&self) -> String {
        format!("group:{id}", id=self.id)
    }
}
