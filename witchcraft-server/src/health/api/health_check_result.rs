use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use conjure_object::serde::{de, ser};
use std::fmt;
#[doc = "Metadata describing the status of a service."]
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct HealthCheckResult {
    type_: super::CheckType,
    state: super::HealthState,
    message: Option<String>,
    params: std::collections::BTreeMap<String, conjure_object::Any>,
}
impl HealthCheckResult {
    #[doc = r" Returns a new builder."]
    #[inline]
    pub fn builder() -> BuilderStage0 {
        Default::default()
    }
    #[doc = "A constant representing the type of health check. Values should be uppercase, underscore delimited, ascii letters with no spaces, ([A-Z_])."]
    #[inline]
    pub fn type_(&self) -> &super::CheckType {
        &self.type_
    }
    #[doc = "Health state of the check."]
    #[inline]
    pub fn state(&self) -> &super::HealthState {
        &self.state
    }
    #[doc = "Text describing the state of the check which should provide enough information for the check to be actionable when included in an alert."]
    #[inline]
    pub fn message(&self) -> Option<&str> {
        self.message.as_ref().map(|o| &**o)
    }
    #[doc = "Additional redacted information on the nature of the health check."]
    #[inline]
    pub fn params(&self) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.params
    }
}
impl Default for BuilderStage0 {
    #[inline]
    fn default() -> Self {
        BuilderStage0 {}
    }
}
impl From<HealthCheckResult> for BuilderStage2 {
    #[inline]
    fn from(value: HealthCheckResult) -> Self {
        BuilderStage2 {
            type_: value.type_,
            state: value.state,
            message: value.message,
            params: value.params,
        }
    }
}
#[doc = "The stage 0 builder for the [`HealthCheckResult`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage0 {}
impl BuilderStage0 {
    #[doc = "A constant representing the type of health check. Values should be uppercase, underscore delimited, ascii letters with no spaces, ([A-Z_])."]
    #[inline]
    pub fn type_(self, type_: super::CheckType) -> BuilderStage1 {
        BuilderStage1 { type_: type_ }
    }
}
#[doc = "The stage 1 builder for the [`HealthCheckResult`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage1 {
    type_: super::CheckType,
}
impl BuilderStage1 {
    #[doc = "Health state of the check."]
    #[inline]
    pub fn state(self, state: super::HealthState) -> BuilderStage2 {
        BuilderStage2 {
            type_: self.type_,
            state: state,
            message: Default::default(),
            params: Default::default(),
        }
    }
}
#[doc = "The stage 2 builder for the [`HealthCheckResult`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage2 {
    type_: super::CheckType,
    state: super::HealthState,
    message: Option<String>,
    params: std::collections::BTreeMap<String, conjure_object::Any>,
}
impl BuilderStage2 {
    #[doc = "A constant representing the type of health check. Values should be uppercase, underscore delimited, ascii letters with no spaces, ([A-Z_])."]
    #[inline]
    pub fn type_(mut self, type_: super::CheckType) -> Self {
        self.type_ = type_;
        self
    }
    #[doc = "Health state of the check."]
    #[inline]
    pub fn state(mut self, state: super::HealthState) -> Self {
        self.state = state;
        self
    }
    #[doc = "Text describing the state of the check which should provide enough information for the check to be actionable when included in an alert."]
    #[inline]
    pub fn message<T>(mut self, message: T) -> Self
    where
        T: Into<Option<String>>,
    {
        self.message = message.into();
        self
    }
    #[doc = "Additional redacted information on the nature of the health check."]
    #[inline]
    pub fn params<T>(mut self, params: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.params = params.into_iter().collect();
        self
    }
    #[doc = "Additional redacted information on the nature of the health check."]
    #[inline]
    pub fn extend_params<T>(mut self, params: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.params.extend(params);
        self
    }
    #[doc = "Additional redacted information on the nature of the health check."]
    #[inline]
    pub fn insert_params<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: conjure_object::serde::Serialize,
    {
        self.params.insert(
            key.into(),
            conjure_object::Any::new(value).expect("value failed to serialize"),
        );
        self
    }
    #[doc = r" Consumes the builder, constructing a new instance of the type."]
    #[inline]
    pub fn build(self) -> HealthCheckResult {
        HealthCheckResult {
            type_: self.type_,
            state: self.state,
            message: self.message,
            params: self.params,
        }
    }
}
impl ser::Serialize for HealthCheckResult {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut size = 2usize;
        let skip_message = self.message.is_none();
        if !skip_message {
            size += 1;
        }
        let skip_params = self.params.is_empty();
        if !skip_params {
            size += 1;
        }
        let mut s = s.serialize_struct("HealthCheckResult", size)?;
        s.serialize_field("type", &self.type_)?;
        s.serialize_field("state", &self.state)?;
        if skip_message {
            s.skip_field("message")?;
        } else {
            s.serialize_field("message", &self.message)?;
        }
        if skip_params {
            s.skip_field("params")?;
        } else {
            s.serialize_field("params", &self.params)?;
        }
        s.end()
    }
}
impl<'de> de::Deserialize<'de> for HealthCheckResult {
    fn deserialize<D>(d: D) -> Result<HealthCheckResult, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_struct(
            "HealthCheckResult",
            &["type", "state", "message", "params"],
            Visitor_,
        )
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = HealthCheckResult;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("map")
    }
    fn visit_map<A>(self, mut map_: A) -> Result<HealthCheckResult, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut type_ = None;
        let mut state = None;
        let mut message = None;
        let mut params = None;
        while let Some(field_) = map_.next_key()? {
            match field_ {
                Field_::Type => type_ = Some(map_.next_value()?),
                Field_::State => state = Some(map_.next_value()?),
                Field_::Message => message = Some(map_.next_value()?),
                Field_::Params => params = Some(map_.next_value()?),
                Field_::Unknown_ => {
                    map_.next_value::<de::IgnoredAny>()?;
                }
            }
        }
        let type_ = match type_ {
            Some(v) => v,
            None => return Err(de::Error::missing_field("type")),
        };
        let state = match state {
            Some(v) => v,
            None => return Err(de::Error::missing_field("state")),
        };
        let message = match message {
            Some(v) => v,
            None => Default::default(),
        };
        let params = match params {
            Some(v) => v,
            None => Default::default(),
        };
        Ok(HealthCheckResult {
            type_,
            state,
            message,
            params,
        })
    }
}
enum Field_ {
    Type,
    State,
    Message,
    Params,
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
            "state" => Field_::State,
            "message" => Field_::Message,
            "params" => Field_::Params,
            _ => Field_::Unknown_,
        };
        Ok(v)
    }
}
