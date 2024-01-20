use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct SetPilot {
    pub method: String,
    pub params: SetPilotParams,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SetPilotParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temp: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimming: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub g: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub b: Option<u32>,
}

impl SetPilot {
    pub fn brightness(&mut self, b: u32) {
        self.params.dimming = Some(b);
    }

    pub fn on(&mut self) {
        self.params.state = Some(true);
    }

    pub fn off(&mut self) -> self {
        self.params.state = Some(false);
        self
    }

    pub fn temperature(&mut self, t: u32) {
        self.params.temp = Some(t);
    }
}

impl Default for SetPilot {
    fn default() -> Self {
        SetPilot {
            method: "setPilot".to_string(),
            params: SetPilotParams {..Default::default()}
        }
    }
}

impl Default for SetPilotParams {
    fn default() -> Self {
        SetPilotParams {
            state: None,
            temp: None,
            dimming: None,
            r: None,
            g: None,
            b: None,
        }
    }
}
