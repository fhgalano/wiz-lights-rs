use std::default::Default;
use std::io::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::str::from_utf8;

use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json;

pub use crate::function::{Off, On};
use crate::utils::ip_addr_ser;
use method::*;
use response::*;
use sourced_response::SourcedResponse;

pub(crate) mod method;
pub mod response;
pub mod sourced_response;

/// we really need to fix the serialization for the IpAddr
/// This solution definitely works: https://github.com/surrealdb/surrealdb/issues/3301#issuecomment-1890672975
/// so I should either learn how this works (i.e. implement the (de)serialize logic myself
/// or try another solution, like using Ipv4Addr for everything
/// I think I could also figure out a deserialization method for maps to help with surreal
/// but I think the solution in the link above is the best one since surreal is jank AF apparently
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Bulb {
    #[serde(with = "ip_addr_ser")]
    ip_address: IpAddr,
    pub _id: u32,
    pub name: String,
    pub state: bool, // tbd
}

impl Bulb {
    pub fn new(ip_address: IpAddr, name: String, id: u32) -> Bulb {
        Bulb {
            ip_address,
            _id: id, // fixme
            name,
            state: false, // fixme
        }
    }

    pub fn get_state(&self) -> Result<bool, ErrorResponse> {
        Ok(self.get_pilot()?.result.state)
    }

    pub fn get_pilot(&self) -> Result<GetPilotResponse, ErrorResponse> {
        let message = serde_json::to_string(&GetPilot::default()).unwrap();

        self.send_message(message.as_bytes()).get_response()
    }

    pub fn set_pilot(&self, p: SetPilot) -> Result<SetPilotResponse, ErrorResponse> {
        let m = serde_json::to_string(&p).unwrap();
        let message: &[u8] = m.as_bytes();

        self.send_message(message).set_response()
    }

    fn send_message(&self, message: &[u8]) -> Response {
        match Self::_send_message(self.ip_address, message) {
            Ok(r) => r,
            Err(e) => {
                error!("Error in UDP Communication {}", e);
                Response::ER(ErrorResponse::default())
            }
        }
    }

    fn _send_message(ip: IpAddr, message: &[u8]) -> anyhow::Result<Response> {
        let sock = give_socket()?;
        let mut buff = [0; 512];

        sock.send_to(message, SocketAddr::new(ip, 38899));

        Ok(Self::_poll_response(&sock, &mut buff)?.response)
    }

    fn _poll_response(socket: &UdpSocket, buff: &mut [u8]) -> anyhow::Result<SourcedResponse> {
        let received = socket.recv_from(buff)?;

        info!("from {}", received.1);
        info!(
            "{}",
            from_utf8(&buff[..received.0]).unwrap_or("Error retreiving from buffer")
        );

        Ok(SourcedResponse {
            source: match received.1.ip().clone() {
                IpAddr::V4(ip4) => {
                    let [a, b, c, d] = ip4.octets();
                    Ipv4Addr::new(a, b, c, d)
                }
                IpAddr::V6(ip6) => {
                    return Err(anyhow::anyhow!(
                        "We should never hava an Ipv6 Address: {}",
                        ip6
                    ))
                }
            },
            response: serde_json::from_slice(&buff[..received.0])?,
        })
    }

    fn _poll_messages(ip: IpAddr, message: &[u8]) -> anyhow::Result<Vec<SourcedResponse>> {
        let sock = give_socket()?;
        let mut buff = [0; 512];

        // TODO: is it possible to yield from a rust function? or do I need to make channels w/
        // concurrency

        sock.send_to(message, SocketAddr::new(ip, 38899))?;

        let mut sourced_responses = vec![];
        while let Ok(s) = Self::_poll_response(&sock, &mut buff) {
            sourced_responses.push(s);
        }
        Ok(sourced_responses)
    }

    pub fn discover() -> Vec<Ipv4Addr> {
        let message = serde_json::to_string(&GetPilot::default()).unwrap();
        let ip = IpAddr::V4(Ipv4Addr::BROADCAST);

        Self::_poll_messages(ip, message.as_bytes())
            .unwrap_or(vec![])
            .iter()
            .map(|s| s.source)
            .collect()
    }
}

impl On for Bulb {
    fn on(&mut self) -> Result<bool, ErrorResponse> {
        let response = self.set_pilot(SetPilot {
            method: String::from("setPilot"),
            params: SetPilotParams {
                state: Some(true),
                ..Default::default()
            },
        })?;
        self.state = true;
        dbg!(self.clone());

        Ok(response.result.success)
    }
}

impl Off for Bulb {
    fn off(&mut self) -> Result<bool, ErrorResponse> {
        let response = self.set_pilot(SetPilot {
            method: String::from("setPilot"),
            params: SetPilotParams {
                state: Some(false),
                ..Default::default()
            },
        })?;
        self.state = false;
        dbg!(self.clone());

        Ok(response.result.success)
    }
}

fn give_socket() -> Result<UdpSocket, Error> {
    // let port: u16 = 8080;
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;
    socket.set_read_timeout(Some(std::time::Duration::new(2, 0)))?;
    socket.set_write_timeout(Some(std::time::Duration::new(3, 0)))?;

    Ok(socket)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use rstest::{fixture, rstest};
    use std::thread::sleep;
    use std::time::Duration;

    #[fixture]
    pub fn test_bulb(
        #[default(Ipv4Addr::new(192, 168, 68, 70))] ip: Ipv4Addr,
        #[default(0)] id: u32,
    ) -> Bulb {
        Bulb::new(IpAddr::V4(ip), format!("test_bulb_{}", id), id)
    }

    #[rstest]
    fn test_get_pilot(test_bulb: Bulb) {
        let message = test_bulb.get_pilot();
        println!(
            "{}",
            serde_json::to_string_pretty(&message.unwrap()).unwrap()
        );
        assert!(true);
    }

    #[rstest]
    #[case(
        SetPilot::default(),
        r#"{"Err":{"method":"setPilot","error":{"code":-32600,"message":"Invalid Request"}}}"#
    )]
    #[case(SetPilot { params: SetPilotParams { state: Some(true), ..Default::default()}, ..Default::default()}, r#"{"Ok":{"method":"setPilot","result":{"success":true}}}"#)]
    fn test_set_pilot(test_bulb: Bulb, #[case] method: SetPilot, #[case] expected_message: &str) {
        let mymessage = test_bulb.set_pilot(method);
        assert_eq!(serde_json::to_string(&mymessage).unwrap(), expected_message);
    }

    #[rstest]
    fn test_get_state(test_bulb: Bulb) {
        let _ = test_bulb.set_pilot(SetPilot {
            method: String::from("setPilot"),
            params: SetPilotParams {
                state: Some(true),
                ..Default::default()
            },
        });
        let mut state = test_bulb.get_state();
        assert_eq!(state.unwrap(), true);

        sleep(Duration::new(2, 0));

        let _ = test_bulb.set_pilot(SetPilot {
            method: String::from("setPilot"),
            params: SetPilotParams {
                state: Some(false),
                ..Default::default()
            },
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
    fn test_bulb_on(mut test_bulb: Bulb) {
        assert!(test_bulb.on().unwrap());
    }

    #[rstest]
    fn test_bulb_off(mut test_bulb: Bulb) {
        assert!(test_bulb.off().unwrap());
    }

    #[rstest]
    fn test_discover() {
        let socks = Bulb::discover();
        dbg!(socks);
    }
}
