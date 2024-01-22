use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GetPilotResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimming: Option<u32>,
    pub mac: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temp: Option<u32>,
    pub state: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub g: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub b: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SetPilotResult {
    pub success: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResult {
    pub code: i32,
    pub message: String,
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
pub struct ErrorResponse {
    pub method: String,
    pub error: ErrorResult,
}

impl Default for ErrorResponse {
    fn default() -> Self {
        ErrorResponse {
            method: "unknown".to_owned(),
            error: ErrorResult {
                code: 69,
                message: "unknown error detected".to_owned(),
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Response {
    GR(GetPilotResponse),
    SR(SetPilotResponse),
    ER(ErrorResponse),
}

impl Response {
    pub(crate) fn get_response(self) -> Result<GetPilotResponse, ErrorResponse> {
        match self {
            Response::GR(s) => Ok(s),
            Response::ER(s) => Err(s),
            _ => Err(ErrorResponse::default()),
        }
    }

    pub(crate) fn set_response(self) -> Result<SetPilotResponse, ErrorResponse> {
        match self {
            Response::SR(s) => Ok(s),
            Response::ER(s) => Err(s),
            _ => Err(ErrorResponse::default()),
        }
    }
}

