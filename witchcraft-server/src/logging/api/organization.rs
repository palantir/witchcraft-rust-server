use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Organization {
    id: String,
    reason: String,
}
impl Organization {
    /// Constructs a new instance of the type.
    #[inline]
    pub fn new<T, U>(id: T, reason: U) -> Organization
    where
        T: Into<String>,
        U: Into<String>,
    {
        Organization {
            id: id.into(),
            reason: reason.into(),
        }
    }
    /// Returns a new builder.
    #[inline]
    pub fn builder() -> BuilderStage0 {
        Default::default()
    }
    ///Organization ID. Not exposed to downstream consumers.
    #[inline]
    pub fn id(&self) -> &str {
        &*self.id
    }
    ///Explaination of why this organization was attributed to this log.
    #[inline]
    pub fn reason(&self) -> &str {
        &*self.reason
    }
}
impl Default for BuilderStage0 {
    #[inline]
    fn default() -> Self {
        BuilderStage0 {}
    }
}
impl From<Organization> for BuilderStage2 {
    #[inline]
    fn from(value: Organization) -> Self {
        BuilderStage2 {
            id: value.id,
            reason: value.reason,
        }
    }
}
///The stage 0 builder for the [`Organization`] type
#[derive(Debug, Clone)]
pub struct BuilderStage0 {}
impl BuilderStage0 {
    ///Organization ID. Not exposed to downstream consumers.
    #[inline]
    pub fn id<T>(self, id: T) -> BuilderStage1
    where
        T: Into<String>,
    {
        BuilderStage1 { id: id.into() }
    }
}
///The stage 1 builder for the [`Organization`] type
#[derive(Debug, Clone)]
pub struct BuilderStage1 {
    id: String,
}
impl BuilderStage1 {
    ///Explaination of why this organization was attributed to this log.
    #[inline]
    pub fn reason<T>(self, reason: T) -> BuilderStage2
    where
        T: Into<String>,
    {
        BuilderStage2 {
            id: self.id,
            reason: reason.into(),
        }
    }
}
///The stage 2 builder for the [`Organization`] type
#[derive(Debug, Clone)]
pub struct BuilderStage2 {
    id: String,
    reason: String,
}
impl BuilderStage2 {
    ///Organization ID. Not exposed to downstream consumers.
    #[inline]
    pub fn id<T>(mut self, id: T) -> Self
    where
        T: Into<String>,
    {
        self.id = id.into();
        self
    }
    ///Explaination of why this organization was attributed to this log.
    #[inline]
    pub fn reason<T>(mut self, reason: T) -> Self
    where
        T: Into<String>,
    {
        self.reason = reason.into();
        self
    }
    /// Consumes the builder, constructing a new instance of the type.
    #[inline]
    pub fn build(self) -> Organization {
        Organization {
            id: self.id,
            reason: self.reason,
        }
    }
}
impl ser::Serialize for Organization {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let size = 2usize;
        let mut s = s.serialize_struct("Organization", size)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("reason", &self.reason)?;
        s.end()
    }
}
impl<'de> de::Deserialize<'de> for Organization {
    fn deserialize<D>(d: D) -> Result<Organization, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_struct("Organization", &["id", "reason"], Visitor_)
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = Organization;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("map")
    }
    fn visit_map<A>(self, mut map_: A) -> Result<Organization, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut id = None;
        let mut reason = None;
        while let Some(field_) = map_.next_key()? {
            match field_ {
                Field_::Id => id = Some(map_.next_value()?),
                Field_::Reason => reason = Some(map_.next_value()?),
                Field_::Unknown_ => {
                    map_.next_value::<de::IgnoredAny>()?;
                }
            }
        }
        let id = match id {
            Some(v) => v,
            None => return Err(de::Error::missing_field("id")),
        };
        let reason = match reason {
            Some(v) => v,
            None => return Err(de::Error::missing_field("reason")),
        };
        Ok(Organization { id, reason })
    }
}
enum Field_ {
    Id,
    Reason,
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
            "id" => Field_::Id,
            "reason" => Field_::Reason,
            _ => Field_::Unknown_,
        };
        Ok(v)
    }
}
