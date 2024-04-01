use async_trait::async_trait;
use crate::bulb::Bulb;
use crate::utils::surreal::GraphLink;

#[async_trait]
#[typetag::serde]
impl GraphLink for Bulb {
    fn query_id_string(&self) -> String {
        format!("bulb:{id}", id=self._id)
    }
}
