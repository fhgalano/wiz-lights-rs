use std::io::Error;
use std::net::{IpAddr, UdpSocket, SocketAddr, Ipv4Addr};
use std::str::from_utf8;
use std::default::Default;

use serde::{Deserialize, Serialize};
use serde_json;

use crate::utils::ip_addr_ser;
use response::*;
use method::*;
pub use crate::function::{Off, On};

pub mod response;
mod method;


/// we really need to fix the serialization for the IpAddr
/// This solution definitely works: https://github.com/surrealdb/surrealdb/issues/3301#issuecomment-1890672975
/// so I should either learn how this works (i.e. implement the (de)serialize logic myself
/// or try another solution, like using Ipv4Addr for everything
/// I think I could also figure out a deserialization method for maps to help with surreal
/// but I think the solution in the link above is the best one since surreal is jank AF apparently
#[derive(
    Debug, PartialEq, Clone,
    Serialize, Deserialize
)]
pub struct Bulb {
    #[serde(with = "ip_addr_ser")]
    ip_address: IpAddr,
    pub _id: u32,
    name: String,
    state: bool, // tbd
}

impl Bulb {
    pub(crate) fn new(ip_address: IpAddr, name: String, id: u32) -> Bulb {
        Bulb {
            ip_address,
            _id: id, // fixme
            name,
            state: false, // fixme
        }
    }

    fn get_state(&self) -> Result<bool, ErrorResponse> {
        Ok(self.get_pilot()?.result.state)
    }

    fn get_pilot(&self) -> Result<GetPilotResponse, ErrorResponse> {
        let message = serde_json::to_string(&GetPilot::default()).unwrap();

        self.send_message(message.as_bytes()).get_response()
    }

    fn set_pilot(&self, p: SetPilot) -> Result<SetPilotResponse, ErrorResponse> {
        let m = serde_json::to_string(&p).unwrap();
        let message: &[u8] = m.as_bytes();

        self.send_message(message).set_response()
    }

    fn send_message(&self, message: &[u8]) -> Response {
        let sock = give_socket().unwrap();
        let mut buff = [0; 512];

        let _ = sock.send_to(message, SocketAddr::new(self.ip_address, 38899));

        match sock.recv_from(&mut buff) {
            Ok(received) => {
                println!("from {}", received.1);
                println!("{}", from_utf8(&buff[..received.0]).unwrap());
                serde_json::from_slice(&buff[..received.0]).unwrap()
            },
            Err(e) => {
                panic!("recv function failed: {e:?}");
            },
        }
    }
}

impl On for Bulb {
    fn on(&self) -> Result<bool, ErrorResponse> {
        let response = self.set_pilot(SetPilot {
            method: String::from("setPilot"),
            params: SetPilotParams {
                state: Some(true),
                ..Default::default()
            }
        })?;

        Ok(response.result.success)
    }
}

impl Off for Bulb {
    fn off(&self) -> Result<bool, ErrorResponse> {
        let response = self.set_pilot(SetPilot {
            method: String::from("setPilot"),
            params: SetPilotParams {
                state: Some(false),
                ..Default::default()
            }
        })?;

        Ok(response.result.success)
    }
}

fn give_socket() -> Result<UdpSocket, Error> {
    // let port: u16 = 8080;
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;
    // .set_read_timeout(Duration::new(1, 0))?;

    Ok(socket)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use rstest::{rstest, fixture};
    use std::time::Duration;
    use std::thread::sleep;

    #[fixture]
    pub fn test_bulb(
        #[default(Ipv4Addr::new(192, 168, 68, 54))]
        ip: Ipv4Addr,
        #[default(0)]
        id: u32,
    ) -> Bulb {
        Bulb::new(
            IpAddr::V4(ip),
            format!("test_bulb_{}", id),
            id,
        )
    }

    #[rstest]
    fn test_get_pilot(test_bulb: Bulb) {
        let message = test_bulb.get_pilot();
        println!("{}", serde_json::to_string_pretty(&message.unwrap()).unwrap());
        assert!(true);
    }

    #[rstest]
    #[case(SetPilot::default(), r#"{"Err":{"method":"setPilot","error":{"code":-32600,"message":"Invalid Request"}}}"#)]
    #[case(SetPilot { params: SetPilotParams { state: Some(true), ..Default::default()}, ..Default::default()}, r#"{"Ok":{"method":"setPilot","result":{"success":true}}}"#)]
    fn test_set_pilot(
        test_bulb: Bulb,
        #[case] method: SetPilot,
        #[case] expected_message: &str
    ) {
        let mymessage = test_bulb.set_pilot(method);
        assert_eq!(
            serde_json::to_string(&mymessage).unwrap(),
            expected_message
        );
    }

    #[rstest]
    fn test_get_state(test_bulb: Bulb) {
        let _ = test_bulb.set_pilot(SetPilot {
            method: String::from("setPilot"),
            params: SetPilotParams {
                state: Some(true),
                ..Default::default()
            }
        });
        let mut state = test_bulb.get_state();
        assert_eq!(state.unwrap(), true);

        sleep(Duration::new(2, 0));

        let _ = test_bulb.set_pilot(SetPilot {
            method: String::from("setPilot"),
            params: SetPilotParams {
                state: Some(false),
                ..Default::default()
            }
        });
        state = test_bulb.get_state();
        assert_eq!(state.unwrap(), false);
    }

    #[rstest]
    fn test_deserialize_bulb(test_bulb: Bulb) {
        println!("{}", serde_json::to_string(&test_bulb).unwrap());

    }

    #[rstest]
    fn test_serialize_bulb(test_bulb: Bulb) {
        let ser_bulb = serde_json::to_string(&test_bulb).unwrap();
        dbg!(serde_json::from_str::<Bulb>(ser_bulb.as_str()).unwrap());
    }

    #[rstest]
    fn test_bulb_on(test_bulb: Bulb) {
        test_bulb.on();
    }

    #[rstest]
    fn test_bulb_off(test_bulb: Bulb) {
        test_bulb.off();
    }
}
