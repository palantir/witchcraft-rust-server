use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SensitivityTaggedValue {
    level: Vec<String>,
    payload: conjure_object::Any,
}
impl SensitivityTaggedValue {
    /// Constructs a new instance of the type.
    #[inline]
    pub fn new<T, U>(level: T, payload: U) -> SensitivityTaggedValue
    where
        T: IntoIterator<Item = String>,
        U: conjure_object::serde::Serialize,
    {
        SensitivityTaggedValue {
            level: level.into_iter().collect(),
            payload: conjure_object::Any::new(payload)
                .expect("value failed to serialize"),
        }
    }
    /// Returns a new builder.
    #[inline]
    pub fn builder() -> BuilderStage0 {
        Default::default()
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
impl Default for BuilderStage0 {
    #[inline]
    fn default() -> Self {
        BuilderStage0 {}
    }
}
impl From<SensitivityTaggedValue> for BuilderStage1 {
    #[inline]
    fn from(value: SensitivityTaggedValue) -> Self {
        BuilderStage1 {
            level: value.level,
            payload: value.payload,
        }
    }
}
///The stage 0 builder for the [`SensitivityTaggedValue`] type
#[derive(Debug, Clone)]
pub struct BuilderStage0 {}
impl BuilderStage0 {
    #[inline]
    pub fn payload<T>(self, payload: T) -> BuilderStage1
    where
        T: conjure_object::serde::Serialize,
    {
        BuilderStage1 {
            payload: conjure_object::Any::new(payload)
                .expect("value failed to serialize"),
            level: Default::default(),
        }
    }
}
///The stage 1 builder for the [`SensitivityTaggedValue`] type
#[derive(Debug, Clone)]
pub struct BuilderStage1 {
    payload: conjure_object::Any,
    level: Vec<String>,
}
impl BuilderStage1 {
    #[inline]
    pub fn payload<T>(mut self, payload: T) -> Self
    where
        T: conjure_object::serde::Serialize,
    {
        self
            .payload = conjure_object::Any::new(payload)
            .expect("value failed to serialize");
        self
    }
    ///Sensitivity level of this value; must be a known level in sls-spec.
    #[inline]
    pub fn level<T>(mut self, level: T) -> Self
    where
        T: IntoIterator<Item = String>,
    {
        self.level = level.into_iter().collect();
        self
    }
    ///Sensitivity level of this value; must be a known level in sls-spec.
    #[inline]
    pub fn extend_level<T>(mut self, level: T) -> Self
    where
        T: IntoIterator<Item = String>,
    {
        self.level.extend(level);
        self
    }
    ///Sensitivity level of this value; must be a known level in sls-spec.
    #[inline]
    pub fn push_level<T>(mut self, value: T) -> Self
    where
        T: Into<String>,
    {
        self.level.push(value.into());
        self
    }
    /// Consumes the builder, constructing a new instance of the type.
    #[inline]
    pub fn build(self) -> SensitivityTaggedValue {
        SensitivityTaggedValue {
            level: self.level,
            payload: self.payload,
        }
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
