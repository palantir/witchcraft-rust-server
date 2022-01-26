use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use conjure_object::serde::{de, ser};
use std::fmt;
#[doc = "A Zipkin-compatible Annotation object."]
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Annotation {
    timestamp: conjure_object::SafeLong,
    value: String,
    endpoint: Box<super::Endpoint>,
}
impl Annotation {
    #[doc = r" Constructs a new instance of the type."]
    #[inline]
    pub fn new<T>(
        timestamp: conjure_object::SafeLong,
        value: T,
        endpoint: super::Endpoint,
    ) -> Annotation
    where
        T: Into<String>,
    {
        Annotation {
            timestamp: timestamp,
            value: value.into(),
            endpoint: Box::new(endpoint),
        }
    }
    #[doc = r" Returns a new builder."]
    #[inline]
    pub fn builder() -> BuilderStage0 {
        Default::default()
    }
    #[doc = "Time annotation was created (epoch microsecond value)"]
    #[inline]
    pub fn timestamp(&self) -> conjure_object::SafeLong {
        self.timestamp
    }
    #[doc = "Value encapsulated by this annotation"]
    #[inline]
    pub fn value(&self) -> &str {
        &*self.value
    }
    #[inline]
    pub fn endpoint(&self) -> &super::Endpoint {
        &*self.endpoint
    }
}
impl Default for BuilderStage0 {
    #[inline]
    fn default() -> Self {
        BuilderStage0 {}
    }
}
impl From<Annotation> for BuilderStage3 {
    #[inline]
    fn from(value: Annotation) -> Self {
        BuilderStage3 {
            timestamp: value.timestamp,
            value: value.value,
            endpoint: value.endpoint,
        }
    }
}
#[doc = "The stage 0 builder for the [`Annotation`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage0 {}
impl BuilderStage0 {
    #[doc = "Time annotation was created (epoch microsecond value)"]
    #[inline]
    pub fn timestamp(self, timestamp: conjure_object::SafeLong) -> BuilderStage1 {
        BuilderStage1 {
            timestamp: timestamp,
        }
    }
}
#[doc = "The stage 1 builder for the [`Annotation`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage1 {
    timestamp: conjure_object::SafeLong,
}
impl BuilderStage1 {
    #[doc = "Value encapsulated by this annotation"]
    #[inline]
    pub fn value<T>(self, value: T) -> BuilderStage2
    where
        T: Into<String>,
    {
        BuilderStage2 {
            timestamp: self.timestamp,
            value: value.into(),
        }
    }
}
#[doc = "The stage 2 builder for the [`Annotation`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage2 {
    timestamp: conjure_object::SafeLong,
    value: String,
}
impl BuilderStage2 {
    #[inline]
    pub fn endpoint(self, endpoint: super::Endpoint) -> BuilderStage3 {
        BuilderStage3 {
            timestamp: self.timestamp,
            value: self.value,
            endpoint: Box::new(endpoint),
        }
    }
}
#[doc = "The stage 3 builder for the [`Annotation`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage3 {
    timestamp: conjure_object::SafeLong,
    value: String,
    endpoint: Box<super::Endpoint>,
}
impl BuilderStage3 {
    #[doc = "Time annotation was created (epoch microsecond value)"]
    #[inline]
    pub fn timestamp(mut self, timestamp: conjure_object::SafeLong) -> Self {
        self.timestamp = timestamp;
        self
    }
    #[doc = "Value encapsulated by this annotation"]
    #[inline]
    pub fn value<T>(mut self, value: T) -> Self
    where
        T: Into<String>,
    {
        self.value = value.into();
        self
    }
    #[inline]
    pub fn endpoint(mut self, endpoint: super::Endpoint) -> Self {
        self.endpoint = Box::new(endpoint);
        self
    }
    #[doc = r" Consumes the builder, constructing a new instance of the type."]
    #[inline]
    pub fn build(self) -> Annotation {
        Annotation {
            timestamp: self.timestamp,
            value: self.value,
            endpoint: self.endpoint,
        }
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
