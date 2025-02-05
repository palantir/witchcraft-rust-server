#![allow(deprecated)]
use std::fmt;
use std::str;
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    conjure_object::serde::Deserialize,
    conjure_object::serde::Serialize,
)]
#[serde(crate = "conjure_object::serde")]
pub enum AuditProducer {
    #[serde(rename = "SERVER")]
    Server,
    #[serde(rename = "CLIENT")]
    Client,
}
impl AuditProducer {
    /// Returns the string representation of the enum.
    #[inline]
    pub fn as_str(&self) -> &str {
        match self {
            AuditProducer::Server => "SERVER",
            AuditProducer::Client => "CLIENT",
        }
    }
}
impl fmt::Display for AuditProducer {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), fmt)
    }
}
impl conjure_object::Plain for AuditProducer {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        conjure_object::Plain::fmt(self.as_str(), fmt)
    }
}
impl str::FromStr for AuditProducer {
    type Err = conjure_object::plain::ParseEnumError;
    #[inline]
    fn from_str(
        v: &str,
    ) -> Result<AuditProducer, conjure_object::plain::ParseEnumError> {
        match v {
            "SERVER" => Ok(AuditProducer::Server),
            "CLIENT" => Ok(AuditProducer::Client),
            _ => Err(conjure_object::plain::ParseEnumError::new()),
        }
    }
}
impl conjure_object::FromPlain for AuditProducer {
    type Err = conjure_object::plain::ParseEnumError;
    #[inline]
    fn from_plain(
        v: &str,
    ) -> Result<AuditProducer, conjure_object::plain::ParseEnumError> {
        v.parse()
    }
}
