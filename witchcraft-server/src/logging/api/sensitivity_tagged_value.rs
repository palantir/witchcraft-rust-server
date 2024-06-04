use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[conjure_object::private::staged_builder::staged_builder]
#[builder(crate = conjure_object::private::staged_builder, update, inline)]
pub struct SensitivityTaggedValue {
    #[builder(default, list(item(type = String, into)))]
    level: Vec<String>,
    #[builder(
        custom(
            type = impl
            conjure_object::serde::Serialize,
            convert = |v|conjure_object::Any::new(v).expect("value failed to serialize")
        )
    )]
    payload: conjure_object::Any,
}
impl SensitivityTaggedValue {
    /// Constructs a new instance of the type.
    #[inline]
    pub fn new(payload: impl conjure_object::serde::Serialize) -> Self {
        Self::builder().payload(payload).build()
    }
    ///Sensitivity level of this value; must be a known level in sls-spec.
    #[inline]
    pub fn level(&self) -> &[String] {
        &*self.level
    }
    #[inline]
    pub fn payload(&self) -> &conjure_object::Any {
        &self.payload
    }
}
impl ser::Serialize for SensitivityTaggedValue {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut size = 1usize;
        let skip_level = self.level.is_empty();
        if !skip_level {
            size += 1;
        }
        let mut s = s.serialize_struct("SensitivityTaggedValue", size)?;
        if skip_level {
            s.skip_field("level")?;
        } else {
            s.serialize_field("level", &self.level)?;
        }
        s.serialize_field("payload", &self.payload)?;
        s.end()
    }
}
impl<'de> de::Deserialize<'de> for SensitivityTaggedValue {
    fn deserialize<D>(d: D) -> Result<SensitivityTaggedValue, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_struct("SensitivityTaggedValue", &["level", "payload"], Visitor_)
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = SensitivityTaggedValue;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("map")
    }
    fn visit_map<A>(self, mut map_: A) -> Result<SensitivityTaggedValue, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut level = None;
        let mut payload = None;
        while let Some(field_) = map_.next_key()? {
            match field_ {
                Field_::Level => level = Some(map_.next_value()?),
                Field_::Payload => payload = Some(map_.next_value()?),
                Field_::Unknown_ => {
                    map_.next_value::<de::IgnoredAny>()?;
                }
            }
        }
        let level = match level {
            Some(v) => v,
            None => Default::default(),
        };
        let payload = match payload {
            Some(v) => v,
            None => return Err(de::Error::missing_field("payload")),
        };
        Ok(SensitivityTaggedValue {
            level,
            payload,
        })
    }
}
enum Field_ {
    Level,
    Payload,
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
            "level" => Field_::Level,
            "payload" => Field_::Payload,
            _ => Field_::Unknown_,
        };
        Ok(v)
    }
}
