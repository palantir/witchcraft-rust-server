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
pub enum LogLevel {
    #[serde(rename = "FATAL")]
    Fatal,
    #[serde(rename = "ERROR")]
    Error,
    #[serde(rename = "WARN")]
    Warn,
    #[serde(rename = "INFO")]
    Info,
    #[serde(rename = "DEBUG")]
    Debug,
    #[serde(rename = "TRACE")]
    Trace,
}
impl LogLevel {
    /// Returns the string representation of the enum.
    #[inline]
    pub fn as_str(&self) -> &str {
        match self {
            LogLevel::Fatal => "FATAL",
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }
}
impl fmt::Display for LogLevel {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), fmt)
    }
}
impl conjure_object::Plain for LogLevel {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        conjure_object::Plain::fmt(self.as_str(), fmt)
    }
}
impl str::FromStr for LogLevel {
    type Err = conjure_object::plain::ParseEnumError;
    #[inline]
    fn from_str(v: &str) -> Result<LogLevel, conjure_object::plain::ParseEnumError> {
        match v {
            "FATAL" => Ok(LogLevel::Fatal),
            "ERROR" => Ok(LogLevel::Error),
            "WARN" => Ok(LogLevel::Warn),
            "INFO" => Ok(LogLevel::Info),
            "DEBUG" => Ok(LogLevel::Debug),
            "TRACE" => Ok(LogLevel::Trace),
            _ => Err(conjure_object::plain::ParseEnumError::new()),
        }
    }
}
impl conjure_object::FromPlain for LogLevel {
    type Err = conjure_object::plain::ParseEnumError;
    #[inline]
    fn from_plain(v: &str) -> Result<LogLevel, conjure_object::plain::ParseEnumError> {
        v.parse()
    }
}
