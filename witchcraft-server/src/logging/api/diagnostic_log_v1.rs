use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use conjure_object::serde::{de, ser};
use std::fmt;
#[doc = "Definition of the diagnostic.1 format."]
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct DiagnosticLogV1 {
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
    diagnostic: Box<super::Diagnostic>,
    unsafe_params: std::collections::BTreeMap<String, conjure_object::Any>,
}
impl DiagnosticLogV1 {
    #[doc = r" Returns a new builder."]
    #[inline]
    pub fn builder() -> BuilderStage0 {
        Default::default()
    }
    #[doc = "\"diagnostic.1\""]
    #[inline]
    pub fn type_(&self) -> &str {
        &*self.type_
    }
    #[inline]
    pub fn time(&self) -> conjure_object::DateTime<conjure_object::Utc> {
        self.time
    }
    #[doc = "The diagnostic being logged."]
    #[inline]
    pub fn diagnostic(&self) -> &super::Diagnostic {
        &*self.diagnostic
    }
    #[doc = "Unredacted parameters"]
    #[inline]
    pub fn unsafe_params(&self) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.unsafe_params
    }
}
impl Default for BuilderStage0 {
    #[inline]
    fn default() -> Self {
        BuilderStage0 {}
    }
}
impl From<DiagnosticLogV1> for BuilderStage3 {
    #[inline]
    fn from(value: DiagnosticLogV1) -> Self {
        BuilderStage3 {
            type_: value.type_,
            time: value.time,
            diagnostic: value.diagnostic,
            unsafe_params: value.unsafe_params,
        }
    }
}
#[doc = "The stage 0 builder for the [`DiagnosticLogV1`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage0 {}
impl BuilderStage0 {
    #[doc = "\"diagnostic.1\""]
    #[inline]
    pub fn type_<T>(self, type_: T) -> BuilderStage1
    where
        T: Into<String>,
    {
        BuilderStage1 {
            type_: type_.into(),
        }
    }
}
#[doc = "The stage 1 builder for the [`DiagnosticLogV1`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage1 {
    type_: String,
}
impl BuilderStage1 {
    #[inline]
    pub fn time(self, time: conjure_object::DateTime<conjure_object::Utc>) -> BuilderStage2 {
        BuilderStage2 {
            type_: self.type_,
            time: time,
        }
    }
}
#[doc = "The stage 2 builder for the [`DiagnosticLogV1`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage2 {
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
}
impl BuilderStage2 {
    #[doc = "The diagnostic being logged."]
    #[inline]
    pub fn diagnostic(self, diagnostic: super::Diagnostic) -> BuilderStage3 {
        BuilderStage3 {
            type_: self.type_,
            time: self.time,
            diagnostic: Box::new(diagnostic),
            unsafe_params: Default::default(),
        }
    }
}
#[doc = "The stage 3 builder for the [`DiagnosticLogV1`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage3 {
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
    diagnostic: Box<super::Diagnostic>,
    unsafe_params: std::collections::BTreeMap<String, conjure_object::Any>,
}
impl BuilderStage3 {
    #[doc = "\"diagnostic.1\""]
    #[inline]
    pub fn type_<T>(mut self, type_: T) -> Self
    where
        T: Into<String>,
    {
        self.type_ = type_.into();
        self
    }
    #[inline]
    pub fn time(mut self, time: conjure_object::DateTime<conjure_object::Utc>) -> Self {
        self.time = time;
        self
    }
    #[doc = "The diagnostic being logged."]
    #[inline]
    pub fn diagnostic(mut self, diagnostic: super::Diagnostic) -> Self {
        self.diagnostic = Box::new(diagnostic);
        self
    }
    #[doc = "Unredacted parameters"]
    #[inline]
    pub fn unsafe_params<T>(mut self, unsafe_params: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.unsafe_params = unsafe_params.into_iter().collect();
        self
    }
    #[doc = "Unredacted parameters"]
    #[inline]
    pub fn extend_unsafe_params<T>(mut self, unsafe_params: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.unsafe_params.extend(unsafe_params);
        self
    }
    #[doc = "Unredacted parameters"]
    #[inline]
    pub fn insert_unsafe_params<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: conjure_object::serde::Serialize,
    {
        self.unsafe_params.insert(
            key.into(),
            conjure_object::Any::new(value).expect("value failed to serialize"),
        );
        self
    }
    #[doc = r" Consumes the builder, constructing a new instance of the type."]
    #[inline]
    pub fn build(self) -> DiagnosticLogV1 {
        DiagnosticLogV1 {
            type_: self.type_,
            time: self.time,
            diagnostic: self.diagnostic,
            unsafe_params: self.unsafe_params,
        }
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
