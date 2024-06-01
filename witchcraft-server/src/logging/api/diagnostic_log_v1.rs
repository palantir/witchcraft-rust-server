use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
///Definition of the diagnostic.1 format.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[conjure_object::private::staged_builder::staged_builder]
#[builder(crate = conjure_object::private::staged_builder, update, inline)]
pub struct DiagnosticLogV1 {
    #[builder(into)]
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
    #[builder(custom(type = super::Diagnostic, convert = Box::new))]
    diagnostic: Box<super::Diagnostic>,
    #[builder(
        default,
        map(
            key(type = String, into),
            value(
                custom(
                    type = impl
                    conjure_object::serde::Serialize,
                    convert = |v|conjure_object::Any::new(
                        v
                    ).expect("value failed to serialize")
                )
            )
        )
    )]
    unsafe_params: std::collections::BTreeMap<String, conjure_object::Any>,
}
impl DiagnosticLogV1 {
    /// Constructs a new instance of the type.
    #[inline]
    pub fn new(
        type_: impl Into<String>,
        time: conjure_object::DateTime<conjure_object::Utc>,
        diagnostic: super::Diagnostic,
    ) -> Self {
        Self::builder().type_(type_).time(time).diagnostic(diagnostic).build()
    }
    ///"diagnostic.1"
    #[inline]
    pub fn type_(&self) -> &str {
        &*self.type_
    }
    #[inline]
    pub fn time(&self) -> conjure_object::DateTime<conjure_object::Utc> {
        self.time
    }
    ///The diagnostic being logged.
    #[inline]
    pub fn diagnostic(&self) -> &super::Diagnostic {
        &*self.diagnostic
    }
    ///Unredacted parameters
    #[inline]
    pub fn unsafe_params(
        &self,
    ) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.unsafe_params
    }
}
impl ser::Serialize for DiagnosticLogV1 {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut size = 3usize;
        let skip_unsafe_params = self.unsafe_params.is_empty();
        if !skip_unsafe_params {
            size += 1;
        }
        let mut s = s.serialize_struct("DiagnosticLogV1", size)?;
        s.serialize_field("type", &self.type_)?;
        s.serialize_field("time", &self.time)?;
        s.serialize_field("diagnostic", &self.diagnostic)?;
        if skip_unsafe_params {
            s.skip_field("unsafeParams")?;
        } else {
            s.serialize_field("unsafeParams", &self.unsafe_params)?;
        }
        s.end()
    }
}
impl<'de> de::Deserialize<'de> for DiagnosticLogV1 {
    fn deserialize<D>(d: D) -> Result<DiagnosticLogV1, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_struct(
            "DiagnosticLogV1",
            &["type", "time", "diagnostic", "unsafeParams"],
            Visitor_,
        )
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = DiagnosticLogV1;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("map")
    }
    fn visit_map<A>(self, mut map_: A) -> Result<DiagnosticLogV1, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut type_ = None;
        let mut time = None;
        let mut diagnostic = None;
        let mut unsafe_params = None;
        while let Some(field_) = map_.next_key()? {
            match field_ {
                Field_::Type => type_ = Some(map_.next_value()?),
                Field_::Time => time = Some(map_.next_value()?),
                Field_::Diagnostic => diagnostic = Some(map_.next_value()?),
                Field_::UnsafeParams => unsafe_params = Some(map_.next_value()?),
                Field_::Unknown_ => {
                    map_.next_value::<de::IgnoredAny>()?;
                }
            }
        }
        let type_ = match type_ {
            Some(v) => v,
            None => return Err(de::Error::missing_field("type")),
        };
        let time = match time {
            Some(v) => v,
            None => return Err(de::Error::missing_field("time")),
        };
        let diagnostic = match diagnostic {
            Some(v) => v,
            None => return Err(de::Error::missing_field("diagnostic")),
        };
        let unsafe_params = match unsafe_params {
            Some(v) => v,
            None => Default::default(),
        };
        Ok(DiagnosticLogV1 {
            type_,
            time,
            diagnostic,
            unsafe_params,
        })
    }
}
enum Field_ {
    Type,
    Time,
    Diagnostic,
    UnsafeParams,
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
            "type" => Field_::Type,
            "time" => Field_::Time,
            "diagnostic" => Field_::Diagnostic,
            "unsafeParams" => Field_::UnsafeParams,
            _ => Field_::Unknown_,
        };
        Ok(v)
    }
}
