use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use conjure_object::serde::{de, ser};
use std::fmt;
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct GenericDiagnostic {
    diagnostic_type: String,
    value: conjure_object::Any,
}
impl GenericDiagnostic {
    #[doc = r" Constructs a new instance of the type."]
    #[inline]
    pub fn new<T, U>(diagnostic_type: T, value: U) -> GenericDiagnostic
    where
        T: Into<String>,
        U: conjure_object::serde::Serialize,
    {
        GenericDiagnostic {
            diagnostic_type: diagnostic_type.into(),
            value: conjure_object::Any::new(value).expect("value failed to serialize"),
        }
    }
    #[doc = r" Returns a new builder."]
    #[inline]
    pub fn builder() -> BuilderStage0 {
        Default::default()
    }
    #[doc = "An identifier for the type of diagnostic represented."]
    #[inline]
    pub fn diagnostic_type(&self) -> &str {
        &*self.diagnostic_type
    }
    #[doc = "Observations, measurements and context associated with the diagnostic."]
    #[inline]
    pub fn value(&self) -> &conjure_object::Any {
        &self.value
    }
}
impl Default for BuilderStage0 {
    #[inline]
    fn default() -> Self {
        BuilderStage0 {}
    }
}
impl From<GenericDiagnostic> for BuilderStage2 {
    #[inline]
    fn from(value: GenericDiagnostic) -> Self {
        BuilderStage2 {
            diagnostic_type: value.diagnostic_type,
            value: value.value,
        }
    }
}
#[doc = "The stage 0 builder for the [`GenericDiagnostic`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage0 {}
impl BuilderStage0 {
    #[doc = "An identifier for the type of diagnostic represented."]
    #[inline]
    pub fn diagnostic_type<T>(self, diagnostic_type: T) -> BuilderStage1
    where
        T: Into<String>,
    {
        BuilderStage1 {
            diagnostic_type: diagnostic_type.into(),
        }
    }
}
#[doc = "The stage 1 builder for the [`GenericDiagnostic`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage1 {
    diagnostic_type: String,
}
impl BuilderStage1 {
    #[doc = "Observations, measurements and context associated with the diagnostic."]
    #[inline]
    pub fn value<T>(self, value: T) -> BuilderStage2
    where
        T: conjure_object::serde::Serialize,
    {
        BuilderStage2 {
            diagnostic_type: self.diagnostic_type,
            value: conjure_object::Any::new(value).expect("value failed to serialize"),
        }
    }
}
#[doc = "The stage 2 builder for the [`GenericDiagnostic`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage2 {
    diagnostic_type: String,
    value: conjure_object::Any,
}
impl BuilderStage2 {
    #[doc = "An identifier for the type of diagnostic represented."]
    #[inline]
    pub fn diagnostic_type<T>(mut self, diagnostic_type: T) -> Self
    where
        T: Into<String>,
    {
        self.diagnostic_type = diagnostic_type.into();
        self
    }
    #[doc = "Observations, measurements and context associated with the diagnostic."]
    #[inline]
    pub fn value<T>(mut self, value: T) -> Self
    where
        T: conjure_object::serde::Serialize,
    {
        self.value = conjure_object::Any::new(value).expect("value failed to serialize");
        self
    }
    #[doc = r" Consumes the builder, constructing a new instance of the type."]
    #[inline]
    pub fn build(self) -> GenericDiagnostic {
        GenericDiagnostic {
            diagnostic_type: self.diagnostic_type,
            value: self.value,
        }
    }
}
impl ser::Serialize for GenericDiagnostic {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let size = 2usize;
        let mut s = s.serialize_struct("GenericDiagnostic", size)?;
        s.serialize_field("diagnosticType", &self.diagnostic_type)?;
        s.serialize_field("value", &self.value)?;
        s.end()
    }
}
impl<'de> de::Deserialize<'de> for GenericDiagnostic {
    fn deserialize<D>(d: D) -> Result<GenericDiagnostic, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_struct("GenericDiagnostic", &["diagnosticType", "value"], Visitor_)
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = GenericDiagnostic;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("map")
    }
    fn visit_map<A>(self, mut map_: A) -> Result<GenericDiagnostic, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut diagnostic_type = None;
        let mut value = None;
        while let Some(field_) = map_.next_key()? {
            match field_ {
                Field_::DiagnosticType => diagnostic_type = Some(map_.next_value()?),
                Field_::Value => value = Some(map_.next_value()?),
                Field_::Unknown_ => {
                    map_.next_value::<de::IgnoredAny>()?;
                }
            }
        }
        let diagnostic_type = match diagnostic_type {
            Some(v) => v,
            None => return Err(de::Error::missing_field("diagnosticType")),
        };
        let value = match value {
            Some(v) => v,
            None => return Err(de::Error::missing_field("value")),
        };
        Ok(GenericDiagnostic {
            diagnostic_type,
            value,
        })
    }
}
enum Field_ {
    DiagnosticType,
    Value,
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
            "diagnosticType" => Field_::DiagnosticType,
            "value" => Field_::Value,
            _ => Field_::Unknown_,
        };
        Ok(v)
    }
}
