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

    pub fn off(&mut self) -> &mut Self {
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
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use rstest::{rstest, fixture};

    #[rstest]
    #[case(SetPilotParams {state: Some(true), ..Default::default()}, r#"{"method":"setPilot","params":{"state":true}}"#)]
    #[case(SetPilotParams {temp: Some(4000), ..Default::default()}, r#"{"method":"setPilot","params":{"temp":4000}}"#)]
    #[case(SetPilotParams {dimming: Some(80), ..Default::default()}, r#"{"method":"setPilot","params":{"dimming":80}}"#)]
    #[case(SetPilotParams {r: Some(0), ..Default::default()}, r#"{"method":"setPilot","params":{"r":0}}"#)]
    #[case(SetPilotParams {g: Some(128), ..Default::default()}, r#"{"method":"setPilot","params":{"g":128}}"#)]
    #[case(SetPilotParams {b: Some(255), ..Default::default()}, r#"{"method":"setPilot","params":{"b":255}}"#)]
    fn test_set_pilot_serialization(#[case] params: SetPilotParams, #[case] expected_message: &str) {
        let a = SetPilot {
            method: String::from("setPilot"),
            params
        };

        let message = serde_json::to_string(&a).unwrap();

        assert_eq!(
            message,
            expected_message
        );
    }
}