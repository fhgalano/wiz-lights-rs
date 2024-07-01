use crate::bulb::response::Response;
use std::net::Ipv4Addr;

pub struct SourcedResponse {
    pub source: Ipv4Addr,
    pub response: Response,
}
