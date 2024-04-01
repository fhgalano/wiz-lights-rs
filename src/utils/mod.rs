pub mod ip_addr_ser {
    use std::fmt;
    use std::net::IpAddr;
    use serde::{Serializer, Deserializer};
    use serde::de::Visitor;

    pub fn serialize<S>(ip: &IpAddr, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_str(ip.to_string().as_str())
    }

    pub fn deserialize<'de, D>(de: D) -> Result<IpAddr, D::Error>
        where D: Deserializer<'de> {
        struct IpVisitor;

        impl<'de> Visitor<'de> for IpVisitor {
            type Value = IpAddr;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an IpAddr")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where E: serde::de::Error
            {
                Ok(value.parse().unwrap())
            }
        }

        de.deserialize_str(IpVisitor)
    }
}

