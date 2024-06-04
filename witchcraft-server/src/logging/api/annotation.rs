use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
///A Zipkin-compatible Annotation object.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[conjure_object::private::staged_builder::staged_builder]
#[builder(crate = conjure_object::private::staged_builder, update, inline)]
pub struct Annotation {
    timestamp: conjure_object::SafeLong,
    #[builder(into)]
    value: String,
    #[builder(custom(type = super::Endpoint, convert = Box::new))]
    endpoint: Box<super::Endpoint>,
}
impl Annotation {
    /// Constructs a new instance of the type.
    #[inline]
    pub fn new(
        timestamp: conjure_object::SafeLong,
        value: impl Into<String>,
        endpoint: super::Endpoint,
    ) -> Self {
        Self::builder().timestamp(timestamp).value(value).endpoint(endpoint).build()
    }
    ///Time annotation was created (epoch microsecond value)
    #[inline]
    pub fn timestamp(&self) -> conjure_object::SafeLong {
        self.timestamp
    }
    ///Value encapsulated by this annotation
    #[inline]
    pub fn value(&self) -> &str {
        &*self.value
    }
    #[inline]
    pub fn endpoint(&self) -> &super::Endpoint {
        &*self.endpoint
    }
}
impl ser::Serialize for Annotation {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let size = 3usize;
        let mut s = s.serialize_struct("Annotation", size)?;
        s.serialize_field("timestamp", &self.timestamp)?;
        s.serialize_field("value", &self.value)?;
        s.serialize_field("endpoint", &self.endpoint)?;
        s.end()
    }
}
impl<'de> de::Deserialize<'de> for Annotation {
    fn deserialize<D>(d: D) -> Result<Annotation, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_struct("Annotation", &["timestamp", "value", "endpoint"], Visitor_)
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = Annotation;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("map")
    }
    fn visit_map<A>(self, mut map_: A) -> Result<Annotation, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut timestamp = None;
        let mut value = None;
        let mut endpoint = None;
        while let Some(field_) = map_.next_key()? {
            match field_ {
                Field_::Timestamp => timestamp = Some(map_.next_value()?),
                Field_::Value => value = Some(map_.next_value()?),
                Field_::Endpoint => endpoint = Some(map_.next_value()?),
                Field_::Unknown_ => {
                    map_.next_value::<de::IgnoredAny>()?;
                }
            }
        }
        let timestamp = match timestamp {
            Some(v) => v,
            None => return Err(de::Error::missing_field("timestamp")),
        };
        let value = match value {
            Some(v) => v,
            None => return Err(de::Error::missing_field("value")),
        };
        let endpoint = match endpoint {
            Some(v) => v,
            None => return Err(de::Error::missing_field("endpoint")),
        };
        Ok(Annotation {
            timestamp,
            value,
            endpoint,
        })
    }
}
enum Field_ {
    Timestamp,
    Value,
    Endpoint,
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
            "timestamp" => Field_::Timestamp,
            "value" => Field_::Value,
            "endpoint" => Field_::Endpoint,
            _ => Field_::Unknown_,
        };
        Ok(v)
    }
}
