pub use set_methods::{SetPilot};

pub mod get_methods;
pub mod set_methods;

pub enum Method {
    SetPilot(SetPilot)
}
