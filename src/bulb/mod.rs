use std::io::Error;
use std::net::{IpAddr, ToSocketAddrs, UdpSocket, SocketAddr, Ipv4Addr};
use std::str::from_utf8;
use std::default::Default;

use serde::{Serialize, Deserialize, Serializer};
use serde::ser::SerializeStruct;
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

    fn get_state(&self) -> bool {
        self.get_pilot(Some(self.ip_address)).result.state
    }

    fn get_pilot(&self, target: Option<IpAddr>) -> GetPilotResponse {
        let message: &[u8] = r#"{"method":"getPilot","params":{}}"#.as_bytes();

        self.send_message(message).get_response().unwrap()
    }

    fn set_pilot(&self, p: SetPilot) -> SetPilotResponse {
        let m = serde_json::to_string(&p).unwrap();
        let message: &[u8] = m.as_bytes();

        self.send_message(message).set_response().unwrap()
    }

    fn run_method(&self, p: method::Method) -> Response {
        let m = serde_json::to_string(&p).unwrap();
        let message: &[u8] = m.as_bytes();

        self.send_message(message)
    }

    fn send_message(&self, message: &[u8]) -> Response {
        let sock = give_socket().unwrap();
        let mut buff = [0; 512];

        sock.send_to(message, SocketAddr::new(self.ip_address, 38899));

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
    use std::time::Duration;
    use std::thread::sleep;

    //todo: look at the real way to setup fixtures
    fn test_bulb() -> Bulb {
        Bulb::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 68, 64)),
            String::from("tbulb")
        )
    }
    #[test]
    fn checkstuff() {
        let sock = give_socket().unwrap();
        let mut buff = [0; 512];
        let message: &[u8] = r#"{"method":"getPilot","params":{}}"#.as_bytes();
        sock.send_to(message, "192.168.68.64:38899").expect("sup");

        let str_message = match sock.recv(&mut buff) {
            Ok(received) => {
                let m = &buff[..received];
                println!("received {received} bytes {:?}", from_utf8(&buff[..received]));
                m
            },
            Err(e) => {
                println!("recv function failed: {e:?}");
                panic!("sup");
            },
        };

        let message_json: serde_json::Value = serde_json::from_slice(str_message).unwrap();
        // println!("{}", message_json["result"]["state"]);
        println!("{}", serde_json::to_string_pretty(&message_json).unwrap())

    }

    #[test]
    fn test_get_pilot() {
        let tbulb = Bulb::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 68, 64)),
            String::from("tbulb")
        );

        let message = tbulb.get_pilot(None);
        println!("{}", serde_json::to_string_pretty(&message).unwrap());

        let mymessage = tbulb.get_pilot(Some(tbulb.ip_address));
        println!("{}", serde_json::to_string_pretty(&mymessage).unwrap());

        assert!(true);
    }

    #[test]
    fn test_set_pilot() {
        let tbulb = Bulb::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 68, 64)),
            String::from("tbulb")
        );

        let mymessage = tbulb.set_pilot(SetPilot{
            method: String::from("setPilot"),
            params: SetPilotParams {
                state: Some(true),
                ..Default::default()
            }
        });
        println!("{}", serde_json::to_string_pretty(&mymessage).unwrap());

        assert!(true);
    }

    #[test]
    fn test_get_state() {
        let tbulb = Bulb::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 68, 64)),
            String::from("tbulb")
        );

        let _ = tbulb.set_pilot(SetPilot{
            method: String::from("setPilot"),
            params: SetPilotParams {
                state: Some(true),
                ..Default::default()
            }
        });
        let mut state = tbulb.get_state();
        assert_eq!(state, true);

        sleep(Duration::new(2, 0));

        let _ = tbulb.set_pilot(SetPilot{
            method: String::from("setPilot"),
            params: SetPilotParams {
                state: Some(false),
                ..Default::default()
            }
        });
        state = tbulb.get_state();
        assert_eq!(state, false);
    }


    #[test]
    fn compare_raw_strings() {
        let raw_message: &[u8] = r#"{"method":"getPilot","params":{}}"#.as_bytes();

        let mut built = serde_json::json!({
            "method":"getPilot",
            "params":{},
        }).to_string();

        assert_eq!(built.as_bytes(), raw_message);
    }

    #[test]
    fn test_set_pilot_serialization() {
        let a = SetPilot {
            method: String::from("setPilot"),
            params: SetPilotParams {
                state: Some(true),
                temp: None,
                dimming: None,
                r: None,
                g: None,
                b: None,
            }
        };

        let m = serde_json::to_string(&a).unwrap();

        println!("{}", m);

        assert!(true);
    }

    #[test]
    fn test_run_method_set_pilot() {
        let m = Method::SetPilot(SetPilot{..Default::default()}.off());
        let t = test_bulb();

        assert_eq!(
            serde_json::to_string(&t.run_method(m)).unwrap(),
            SetPilotResponse
            )
    }
}
