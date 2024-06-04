use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[conjure_object::private::staged_builder::staged_builder]
#[builder(crate = conjure_object::private::staged_builder, update, inline)]
pub struct Endpoint {
    #[builder(into)]
    service_name: String,
    #[builder(default, into)]
    ipv4: Option<String>,
    #[builder(default, into)]
    ipv6: Option<String>,
}
impl Endpoint {
    /// Constructs a new instance of the type.
    #[inline]
    pub fn new(service_name: impl Into<String>) -> Self {
        Self::builder().service_name(service_name).build()
    }
    ///Name of the service that generated the annotation
    #[inline]
    pub fn service_name(&self) -> &str {
        &*self.service_name
    }
    ///IPv4 address of the machine that generated this annotation (`xxx.xxx.xxx.xxx`)
    #[inline]
    pub fn ipv4(&self) -> Option<&str> {
        self.ipv4.as_ref().map(|o| &**o)
    }
    ///IPv6 address of the machine that generated this annotation (standard hextet form)
    #[inline]
    pub fn ipv6(&self) -> Option<&str> {
        self.ipv6.as_ref().map(|o| &**o)
    }
}
impl ser::Serialize for Endpoint {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut size = 1usize;
        let skip_ipv4 = self.ipv4.is_none();
        if !skip_ipv4 {
            size += 1;
        }
        let skip_ipv6 = self.ipv6.is_none();
        if !skip_ipv6 {
            size += 1;
        }
        let mut s = s.serialize_struct("Endpoint", size)?;
        s.serialize_field("serviceName", &self.service_name)?;
        if skip_ipv4 {
            s.skip_field("ipv4")?;
        } else {
            s.serialize_field("ipv4", &self.ipv4)?;
        }
        if skip_ipv6 {
            s.skip_field("ipv6")?;
        } else {
            s.serialize_field("ipv6", &self.ipv6)?;
        }
        s.end()
    }
}
impl<'de> de::Deserialize<'de> for Endpoint {
    fn deserialize<D>(d: D) -> Result<Endpoint, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_struct("Endpoint", &["serviceName", "ipv4", "ipv6"], Visitor_)
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = Endpoint;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("map")
    }
    fn visit_map<A>(self, mut map_: A) -> Result<Endpoint, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut service_name = None;
        let mut ipv4 = None;
        let mut ipv6 = None;
        while let Some(field_) = map_.next_key()? {
            match field_ {
                Field_::ServiceName => service_name = Some(map_.next_value()?),
                Field_::Ipv4 => ipv4 = Some(map_.next_value()?),
                Field_::Ipv6 => ipv6 = Some(map_.next_value()?),
                Field_::Unknown_ => {
                    map_.next_value::<de::IgnoredAny>()?;
                }
            }
        }
        let service_name = match service_name {
            Some(v) => v,
            None => return Err(de::Error::missing_field("serviceName")),
        };
        let ipv4 = match ipv4 {
            Some(v) => v,
            None => Default::default(),
        };
        let ipv6 = match ipv6 {
            Some(v) => v,
            None => Default::default(),
        };
        Ok(Endpoint {
            service_name,
            ipv4,
            ipv6,
        })
    }
}
enum Field_ {
    ServiceName,
    Ipv4,
    Ipv6,
    Unknown_,
}
impl<'de> de::Deserialize<'de> for Field_ {
    fn deserialize<D>(d: D) -> Result<Field_, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_str(FieldVisitor_)
    }
}
struct FieldVisitor_;
impl<'de> de::Visitor<'de> for FieldVisitor_ {
    type Value = Field_;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("string")
    }
    fn visit_str<E>(self, value: &str) -> Result<Field_, E>
    where
        E: de::Error,
    {
        let v = match value {
            "serviceName" => Field_::ServiceName,
            "ipv4" => Field_::Ipv4,
            "ipv6" => Field_::Ipv6,
            _ => Field_::Unknown_,
        };
        Ok(v)
    }
}
