pub use set_methods::{SetPilot, SetPilotParams};
pub use get_methods::GetPilot;

pub mod get_methods;
pub mod set_methods;

pub enum Method {
    SetPilot(SetPilot)
}
