use conjure_object::serde::{de, ser};
use std::fmt;
use std::str;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LogLevel {
    Fatal,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
impl LogLevel {
    #[doc = r" Returns the string representation of the enum."]
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
impl ser::Serialize for LogLevel {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        s.serialize_str(self.as_str())
    }
}
impl<'de> de::Deserialize<'de> for LogLevel {
    fn deserialize<D>(d: D) -> Result<LogLevel, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_str(Visitor_)
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = LogLevel;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("a string")
    }
    fn visit_str<E>(self, v: &str) -> Result<LogLevel, E>
    where
        E: de::Error,
    {
        match v.parse() {
            Ok(e) => Ok(e),
            Err(_) => Err(de::Error::unknown_variant(
                v,
                &["FATAL", "ERROR", "WARN", "INFO", "DEBUG", "TRACE"],
            )),
        }
    }
}
