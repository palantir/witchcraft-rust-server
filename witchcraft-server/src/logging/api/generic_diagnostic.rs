use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[conjure_object::private::staged_builder::staged_builder]
#[builder(crate = conjure_object::private::staged_builder, update, inline)]
pub struct GenericDiagnostic {
    #[builder(into)]
    diagnostic_type: String,
    #[builder(
        custom(
            type = impl
            conjure_object::serde::Serialize,
            convert = |v|conjure_object::Any::new(v).expect("value failed to serialize")
        )
    )]
    value: conjure_object::Any,
}
impl GenericDiagnostic {
    /// Constructs a new instance of the type.
    #[inline]
    pub fn new(
        diagnostic_type: impl Into<String>,
        value: impl conjure_object::serde::Serialize,
    ) -> Self {
        Self::builder().diagnostic_type(diagnostic_type).value(value).build()
    }
    ///An identifier for the type of diagnostic represented.
    #[inline]
    pub fn diagnostic_type(&self) -> &str {
        &*self.diagnostic_type
    }
    ///Observations, measurements and context associated with the diagnostic.
    #[inline]
    pub fn value(&self) -> &conjure_object::Any {
        &self.value
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
