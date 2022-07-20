use conjure_object::serde::{ser, de};
use std::fmt;
use std::str;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AuditResult {
    Success,
    Unauthorized,
    Error,
}
impl AuditResult {
    /// Returns the string representation of the enum.
    #[inline]
    pub fn as_str(&self) -> &str {
        match self {
            AuditResult::Success => "SUCCESS",
            AuditResult::Unauthorized => "UNAUTHORIZED",
            AuditResult::Error => "ERROR",
        }
    }
}
impl fmt::Display for AuditResult {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), fmt)
    }
}
impl conjure_object::Plain for AuditResult {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        conjure_object::Plain::fmt(self.as_str(), fmt)
    }
}
impl str::FromStr for AuditResult {
    type Err = conjure_object::plain::ParseEnumError;
    #[inline]
    fn from_str(v: &str) -> Result<AuditResult, conjure_object::plain::ParseEnumError> {
        match v {
            "SUCCESS" => Ok(AuditResult::Success),
            "UNAUTHORIZED" => Ok(AuditResult::Unauthorized),
            "ERROR" => Ok(AuditResult::Error),
            _ => Err(conjure_object::plain::ParseEnumError::new()),
        }
    }
}
impl conjure_object::FromPlain for AuditResult {
    type Err = conjure_object::plain::ParseEnumError;
    #[inline]
    fn from_plain(
        v: &str,
    ) -> Result<AuditResult, conjure_object::plain::ParseEnumError> {
        v.parse()
    }
}
impl ser::Serialize for AuditResult {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        s.serialize_str(self.as_str())
    }
}
impl<'de> de::Deserialize<'de> for AuditResult {
    fn deserialize<D>(d: D) -> Result<AuditResult, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_str(Visitor_)
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = AuditResult;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("a string")
    }
    fn visit_str<E>(self, v: &str) -> Result<AuditResult, E>
    where
        E: de::Error,
    {
        match v.parse() {
            Ok(e) => Ok(e),
            Err(_) => {
                Err(de::Error::unknown_variant(v, &["SUCCESS", "UNAUTHORIZED", "ERROR"]))
            }
        }
    }
}
