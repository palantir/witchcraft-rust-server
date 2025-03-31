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
pub enum AuditResult {
    #[serde(rename = "SUCCESS")]
    Success,
    #[serde(rename = "ERROR")]
    Error,
    #[serde(rename = "UNAUTHORIZED")]
    Unauthorized,
    /// A result that has not yet been finalized. It may be missing fields from resultParams, and it is expected that a non-partial log should occur in the future with the same event ID.
    #[serde(rename = "PARTIAL")]
    Partial,
}
impl AuditResult {
    /// Returns the string representation of the enum.
    #[inline]
    pub fn as_str(&self) -> &str {
        match self {
            AuditResult::Success => "SUCCESS",
            AuditResult::Error => "ERROR",
            AuditResult::Unauthorized => "UNAUTHORIZED",
            AuditResult::Partial => "PARTIAL",
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
            "ERROR" => Ok(AuditResult::Error),
            "UNAUTHORIZED" => Ok(AuditResult::Unauthorized),
            "PARTIAL" => Ok(AuditResult::Partial),
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
