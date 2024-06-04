use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
///Metadata describing the status of a service.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[conjure_object::private::staged_builder::staged_builder]
#[builder(crate = conjure_object::private::staged_builder, update, inline)]
pub struct HealthCheckResult {
    type_: super::CheckType,
    state: super::HealthState,
    #[builder(default, into)]
    message: Option<String>,
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
    params: std::collections::BTreeMap<String, conjure_object::Any>,
}
impl HealthCheckResult {
    /// Constructs a new instance of the type.
    #[inline]
    pub fn new(type_: super::CheckType, state: super::HealthState) -> Self {
        Self::builder().type_(type_).state(state).build()
    }
    ///A constant representing the type of health check. Values should be uppercase, underscore delimited, ascii letters with no spaces, ([A-Z_]).
    #[inline]
    pub fn type_(&self) -> &super::CheckType {
        &self.type_
    }
    ///Health state of the check.
    #[inline]
    pub fn state(&self) -> &super::HealthState {
        &self.state
    }
    ///Text describing the state of the check which should provide enough information for the check to be actionable when included in an alert.
    #[inline]
    pub fn message(&self) -> Option<&str> {
        self.message.as_ref().map(|o| &**o)
    }
    ///Additional redacted information on the nature of the health check.
    #[inline]
    pub fn params(&self) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.params
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
