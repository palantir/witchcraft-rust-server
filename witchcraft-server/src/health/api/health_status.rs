use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[conjure_object::private::staged_builder::staged_builder]
#[builder(crate = conjure_object::private::staged_builder, update, inline)]
pub struct HealthStatus {
    #[builder(
        default,
        map(key(type = super::CheckType), value(type = super::HealthCheckResult))
    )]
    checks: std::collections::BTreeMap<super::CheckType, super::HealthCheckResult>,
}
impl HealthStatus {
    /// Constructs a new instance of the type.
    #[inline]
    pub fn new() -> Self {
        Self::builder().build()
    }
    #[inline]
    pub fn checks(
        &self,
    ) -> &std::collections::BTreeMap<super::CheckType, super::HealthCheckResult> {
        &self.checks
    }
}
impl ser::Serialize for HealthStatus {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut size = 0usize;
        let skip_checks = self.checks.is_empty();
        if !skip_checks {
            size += 1;
        }
        let mut s = s.serialize_struct("HealthStatus", size)?;
        if skip_checks {
            s.skip_field("checks")?;
        } else {
            s.serialize_field("checks", &self.checks)?;
        }
        s.end()
    }
}
impl<'de> de::Deserialize<'de> for HealthStatus {
    fn deserialize<D>(d: D) -> Result<HealthStatus, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_struct("HealthStatus", &["checks"], Visitor_)
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = HealthStatus;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("map")
    }
    fn visit_map<A>(self, mut map_: A) -> Result<HealthStatus, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut checks = None;
        while let Some(field_) = map_.next_key()? {
            match field_ {
                Field_::Checks => checks = Some(map_.next_value()?),
                Field_::Unknown_ => {
                    map_.next_value::<de::IgnoredAny>()?;
                }
            }
        }
        let checks = match checks {
            Some(v) => v,
            None => Default::default(),
        };
        Ok(HealthStatus { checks })
    }
}
enum Field_ {
    Checks,
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
            "checks" => Field_::Checks,
            _ => Field_::Unknown_,
        };
        Ok(v)
    }
}
