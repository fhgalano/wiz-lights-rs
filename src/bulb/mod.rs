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
pub(crate) mod method;


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
        Bulb::_send_message(self.ip_address, message)
    }
    
    fn _send_message(ip: IpAddr, message: &[u8]) -> Response {
        let sock = give_socket().unwrap();
        let mut buff = [0; 512];

        let _ = sock.send_to(message, SocketAddr::new(ip, 38899));
        
         match sock.recv_from(&mut buff) {
            Ok(received) => {
                println!("from {}", received.1);
                println!("{}", from_utf8(&buff[..received.0]).unwrap());
                serde_json::from_slice(&buff[..received.0]).unwrap()
            },
            Err(e) => {
                dbg!(e);
                Response::ER(ErrorResponse::default())
            },
        }
    }
    
    pub fn discover() -> Vec<Ipv4Addr>{
        let message = serde_json::to_string(&GetPilot::default()).unwrap();
        let ip = IpAddr::V4(Ipv4Addr::BROADCAST);
        let sock = give_socket().unwrap();
        let mut buff = [0; 512];

        let _ = sock.send_to(message.as_bytes(), SocketAddr::new(ip, 38899));
        let mut addrs: Vec<Ipv4Addr> = vec![];
        while let Ok(received) = sock.recv_from(&mut buff) {
            addrs.push(
                match received.1.ip().clone() {
                    IpAddr::V4(ip4) => {
                        let [a, b, c, d] = ip4.octets();
                        Ipv4Addr::new(a, b, c, d)
                    },
                    IpAddr::V6(ip6) => panic!("We should never hava an Ipv6 Address: {}", ip6)
                }
            );
            info!("from {}", received.1);
            info!("{}", from_utf8(&buff[..received.0]).unwrap());
        };

        addrs
    }
}

impl On for Bulb {
    fn on(&mut self) -> Result<bool, ErrorResponse> {
        let response = self.set_pilot(SetPilot {
            method: String::from("setPilot"),
            params: SetPilotParams {
                state: Some(true),
                ..Default::default()
            }
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
            }
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
    use rstest::{rstest, fixture};
    use std::time::Duration;
    use std::thread::sleep;

    #[fixture]
    pub fn test_bulb(
        #[default(Ipv4Addr::new(192, 168, 68, 58))]
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
