use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GetPilotResult {
    pub dimming: Option<u32>,
    pub mac: String,
    pub temp: u32,
    pub state: bool,
    pub r: Option<u32>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SetPilotResult {
    pub success: bool,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct GetPilotResponse {
    pub method: String,
    pub result: GetPilotResult,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SetPilotResponse {
    pub method: String,
    pub result: SetPilotResult,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Response {
    GR(GetPilotResponse),
    SR(SetPilotResponse),
}

impl Response {
    pub(crate) fn get_response(self) -> Option<GetPilotResponse> {
        match self {
            Response::GR(s) => Some(s),
            _ => None,
        }
    }

    pub(crate) fn set_response(self) -> Option<SetPilotResponse> {
        match self {
            Response::SR(s) => Some(s),
            _ => None,
        }
    }
}

