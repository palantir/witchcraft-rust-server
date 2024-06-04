use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[conjure_object::private::staged_builder::staged_builder]
#[builder(crate = conjure_object::private::staged_builder, update, inline)]
pub struct Organization {
    #[builder(into)]
    id: String,
    #[builder(into)]
    reason: String,
}
impl Organization {
    /// Constructs a new instance of the type.
    #[inline]
    pub fn new(id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::builder().id(id).reason(reason).build()
    }
    ///Organization RID. Not exposed to downstream consumers.
    #[inline]
    pub fn id(&self) -> &str {
        &*self.id
    }
    ///Explanation of why this organization was attributed to this log.
    #[inline]
    pub fn reason(&self) -> &str {
        &*self.reason
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
