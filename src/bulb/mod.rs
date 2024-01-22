use std::io::Error;
use std::net::{IpAddr, UdpSocket, SocketAddr, Ipv4Addr};
use std::str::from_utf8;
use std::default::Default;

use serde_json;

use response::*;
use method::*;

pub mod response;
mod method;


#[derive(PartialEq)]
pub struct Bulb {
    ip_address: IpAddr,
    id: u32,
    name: String,
    state: bool, // tbd
}

impl Bulb {
    fn new(ip_address: IpAddr, name: String) -> Bulb {
        Bulb {
            ip_address,
            id: 12, // fixme
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

fn give_socket() -> Result<UdpSocket, Error> {
    // let port: u16 = 8080;
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;
    // .set_read_timeout(Duration::new(1, 0))?;

    Ok(socket)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{rstest, fixture};
    use std::time::Duration;
    use std::thread::sleep;

    #[fixture]
    fn test_bulb(#[default(Ipv4Addr::new(192, 168, 68, 64))] ip: Ipv4Addr) -> Bulb {
        Bulb::new(
            IpAddr::V4(ip),
            String::from("tbulb")
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
}
