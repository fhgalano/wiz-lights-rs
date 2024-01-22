use serde::Deserialize;
use serde::ser::{Serialize, Serializer, SerializeStruct};


#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct GetPilot {
    pub method: String,
}

#[derive(Deserialize)]
struct GetPilotParams;

impl Serialize for GetPilot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("GetPilot", 2)?;
        state.serialize_field("method", &self.method)?;
        state.serialize_field("params", &GetPilotParams {})?;
        state.end()
    }
}

impl Serialize for GetPilotParams {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        // 3 is the number of fields in the struct.
        let state = serializer.serialize_struct("GetPilotParams", 0)?;
        state.end()
    }
}

impl Default for GetPilot {
    fn default() -> Self {
        GetPilot {
            method: "getPilot".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde_json::to_string;
    #[rstest]
    fn test_get_pilot_serialization() {
        let a = GetPilot{ ..Default::default() };

        assert_eq!(
            to_string(&a).unwrap(),
            r#"{"method":"getPilot","params":{}}"#
        )
    }
}
