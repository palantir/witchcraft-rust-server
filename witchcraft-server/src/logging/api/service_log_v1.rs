use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
///Definition of the service.1 format.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ServiceLogV1 {
    type_: String,
    level: super::LogLevel,
    time: conjure_object::DateTime<conjure_object::Utc>,
    origin: Option<String>,
    thread: Option<String>,
    message: String,
    safe: Option<bool>,
    params: std::collections::BTreeMap<String, conjure_object::Any>,
    uid: Option<super::UserId>,
    sid: Option<super::SessionId>,
    token_id: Option<super::TokenId>,
    trace_id: Option<super::TraceId>,
    stacktrace: Option<String>,
    unsafe_params: std::collections::BTreeMap<String, conjure_object::Any>,
    tags: std::collections::BTreeMap<String, String>,
}
impl ServiceLogV1 {
    /// Returns a new builder.
    #[inline]
    pub fn builder() -> BuilderStage0 {
        Default::default()
    }
    ///"service.1"
    #[inline]
    pub fn type_(&self) -> &str {
        &*self.type_
    }
    ///The logger output level. One of {FATAL,ERROR,WARN,INFO,DEBUG,TRACE} based on [log level coding guidelines](https://github.com/palantir/gradle-baseline/blob/develop/docs/best-practices/java-coding-guidelines/readme.md#log-levels)
    #[inline]
    pub fn level(&self) -> &super::LogLevel {
        &self.level
    }
    ///RFC3339Nano UTC datetime string when the log event was emitted
    #[inline]
    pub fn time(&self) -> conjure_object::DateTime<conjure_object::Utc> {
        self.time
    }
    ///Class or file name. May include line number.
    #[inline]
    pub fn origin(&self) -> Option<&str> {
        self.origin.as_ref().map(|o| &**o)
    }
    ///Thread name
    #[inline]
    pub fn thread(&self) -> Option<&str> {
        self.thread.as_ref().map(|o| &**o)
    }
    ///Log message. Palantir Java services using slf4j should not use slf4j placeholders ({}). Logs obtained from 3rd party libraries or services that use slf4j and contain slf4j placeholders will always produce `unsafeParams` with numeric indexes corresponding to the zero-indexed order of placeholders. Renderers should substitute numeric parameters from `unsafeParams` and may leave placeholders that do not match indexes as the original placeholder text.
    #[inline]
    pub fn message(&self) -> &str {
        &*self.message
    }
    ///Describes the safety of this log event based on prior knowledge within the application which produced the message. This field should not be set to `true` without _total_ confidence that it is correct. * _empty_:  Considered unsafe unless the logging pipeline has special configuration for this `origin`. Eventually these will all be equivalent to `false`. * `true`: All safe components can be trusted. * `false`: Event is _unsafe_ and cannot be exported.
    #[inline]
    pub fn safe(&self) -> Option<bool> {
        self.safe.as_ref().map(|o| *o)
    }
    ///Known-safe parameters (redaction may be used to make params knowably safe, but is not required).
    #[inline]
    pub fn params(&self) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.params
    }
    ///User id (if available).
    #[inline]
    pub fn uid(&self) -> Option<&super::UserId> {
        self.uid.as_ref().map(|o| &*o)
    }
    ///Session id (if available)
    #[inline]
    pub fn sid(&self) -> Option<&super::SessionId> {
        self.sid.as_ref().map(|o| &*o)
    }
    ///API token id (if available)
    #[inline]
    pub fn token_id(&self) -> Option<&super::TokenId> {
        self.token_id.as_ref().map(|o| &*o)
    }
    ///Zipkin trace id (if available)
    #[inline]
    pub fn trace_id(&self) -> Option<&super::TraceId> {
        self.trace_id.as_ref().map(|o| &*o)
    }
    ///Language-specific stack trace. Content is knowably safe. Renderers should substitute named placeholders ({name}, for name as a key) with keyed value from unsafeParams and leave non-matching keys as the original placeholder text.
    #[inline]
    pub fn stacktrace(&self) -> Option<&str> {
        self.stacktrace.as_ref().map(|o| &**o)
    }
    ///Unredacted parameters
    #[inline]
    pub fn unsafe_params(
        &self,
    ) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.unsafe_params
    }
    ///Additional dimensions that describe the instance of the log event
    #[inline]
    pub fn tags(&self) -> &std::collections::BTreeMap<String, String> {
        &self.tags
    }
}
impl Default for BuilderStage0 {
    #[inline]
    fn default() -> Self {
        BuilderStage0 {}
    }
}
impl From<ServiceLogV1> for BuilderStage4 {
    #[inline]
    fn from(value: ServiceLogV1) -> Self {
        BuilderStage4 {
            type_: value.type_,
            level: value.level,
            time: value.time,
            origin: value.origin,
            thread: value.thread,
            message: value.message,
            safe: value.safe,
            params: value.params,
            uid: value.uid,
            sid: value.sid,
            token_id: value.token_id,
            trace_id: value.trace_id,
            stacktrace: value.stacktrace,
            unsafe_params: value.unsafe_params,
            tags: value.tags,
        }
    }
}
///The stage 0 builder for the [`ServiceLogV1`] type
#[derive(Debug, Clone)]
pub struct BuilderStage0 {}
impl BuilderStage0 {
    ///"service.1"
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
///The stage 1 builder for the [`ServiceLogV1`] type
#[derive(Debug, Clone)]
pub struct BuilderStage1 {
    type_: String,
}
impl BuilderStage1 {
    ///The logger output level. One of {FATAL,ERROR,WARN,INFO,DEBUG,TRACE} based on [log level coding guidelines](https://github.com/palantir/gradle-baseline/blob/develop/docs/best-practices/java-coding-guidelines/readme.md#log-levels)
    #[inline]
    pub fn level(self, level: super::LogLevel) -> BuilderStage2 {
        BuilderStage2 {
            type_: self.type_,
            level: level,
        }
    }
}
///The stage 2 builder for the [`ServiceLogV1`] type
#[derive(Debug, Clone)]
pub struct BuilderStage2 {
    type_: String,
    level: super::LogLevel,
}
impl BuilderStage2 {
    ///RFC3339Nano UTC datetime string when the log event was emitted
    #[inline]
    pub fn time(
        self,
        time: conjure_object::DateTime<conjure_object::Utc>,
    ) -> BuilderStage3 {
        BuilderStage3 {
            type_: self.type_,
            level: self.level,
            time: time,
        }
    }
}
///The stage 3 builder for the [`ServiceLogV1`] type
#[derive(Debug, Clone)]
pub struct BuilderStage3 {
    type_: String,
    level: super::LogLevel,
    time: conjure_object::DateTime<conjure_object::Utc>,
}
impl BuilderStage3 {
    ///Log message. Palantir Java services using slf4j should not use slf4j placeholders ({}). Logs obtained from 3rd party libraries or services that use slf4j and contain slf4j placeholders will always produce `unsafeParams` with numeric indexes corresponding to the zero-indexed order of placeholders. Renderers should substitute numeric parameters from `unsafeParams` and may leave placeholders that do not match indexes as the original placeholder text.
    #[inline]
    pub fn message<T>(self, message: T) -> BuilderStage4
    where
        T: Into<String>,
    {
        BuilderStage4 {
            type_: self.type_,
            level: self.level,
            time: self.time,
            message: message.into(),
            origin: Default::default(),
            thread: Default::default(),
            safe: Default::default(),
            params: Default::default(),
            uid: Default::default(),
            sid: Default::default(),
            token_id: Default::default(),
            trace_id: Default::default(),
            stacktrace: Default::default(),
            unsafe_params: Default::default(),
            tags: Default::default(),
        }
    }
}
///The stage 4 builder for the [`ServiceLogV1`] type
#[derive(Debug, Clone)]
pub struct BuilderStage4 {
    type_: String,
    level: super::LogLevel,
    time: conjure_object::DateTime<conjure_object::Utc>,
    message: String,
    origin: Option<String>,
    thread: Option<String>,
    safe: Option<bool>,
    params: std::collections::BTreeMap<String, conjure_object::Any>,
    uid: Option<super::UserId>,
    sid: Option<super::SessionId>,
    token_id: Option<super::TokenId>,
    trace_id: Option<super::TraceId>,
    stacktrace: Option<String>,
    unsafe_params: std::collections::BTreeMap<String, conjure_object::Any>,
    tags: std::collections::BTreeMap<String, String>,
}
impl BuilderStage4 {
    ///"service.1"
    #[inline]
    pub fn type_<T>(mut self, type_: T) -> Self
    where
        T: Into<String>,
    {
        self.type_ = type_.into();
        self
    }
    ///The logger output level. One of {FATAL,ERROR,WARN,INFO,DEBUG,TRACE} based on [log level coding guidelines](https://github.com/palantir/gradle-baseline/blob/develop/docs/best-practices/java-coding-guidelines/readme.md#log-levels)
    #[inline]
    pub fn level(mut self, level: super::LogLevel) -> Self {
        self.level = level;
        self
    }
    ///RFC3339Nano UTC datetime string when the log event was emitted
    #[inline]
    pub fn time(mut self, time: conjure_object::DateTime<conjure_object::Utc>) -> Self {
        self.time = time;
        self
    }
    ///Log message. Palantir Java services using slf4j should not use slf4j placeholders ({}). Logs obtained from 3rd party libraries or services that use slf4j and contain slf4j placeholders will always produce `unsafeParams` with numeric indexes corresponding to the zero-indexed order of placeholders. Renderers should substitute numeric parameters from `unsafeParams` and may leave placeholders that do not match indexes as the original placeholder text.
    #[inline]
    pub fn message<T>(mut self, message: T) -> Self
    where
        T: Into<String>,
    {
        self.message = message.into();
        self
    }
    ///Class or file name. May include line number.
    #[inline]
    pub fn origin<T>(mut self, origin: T) -> Self
    where
        T: Into<Option<String>>,
    {
        self.origin = origin.into();
        self
    }
    ///Thread name
    #[inline]
    pub fn thread<T>(mut self, thread: T) -> Self
    where
        T: Into<Option<String>>,
    {
        self.thread = thread.into();
        self
    }
    ///Describes the safety of this log event based on prior knowledge within the application which produced the message. This field should not be set to `true` without _total_ confidence that it is correct. * _empty_:  Considered unsafe unless the logging pipeline has special configuration for this `origin`. Eventually these will all be equivalent to `false`. * `true`: All safe components can be trusted. * `false`: Event is _unsafe_ and cannot be exported.
    #[inline]
    pub fn safe<T>(mut self, safe: T) -> Self
    where
        T: Into<Option<bool>>,
    {
        self.safe = safe.into();
        self
    }
    ///Known-safe parameters (redaction may be used to make params knowably safe, but is not required).
    #[inline]
    pub fn params<T>(mut self, params: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.params = params.into_iter().collect();
        self
    }
    ///Known-safe parameters (redaction may be used to make params knowably safe, but is not required).
    #[inline]
    pub fn extend_params<T>(mut self, params: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.params.extend(params);
        self
    }
    ///Known-safe parameters (redaction may be used to make params knowably safe, but is not required).
    #[inline]
    pub fn insert_params<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: conjure_object::serde::Serialize,
    {
        self.params
            .insert(
                key.into(),
                conjure_object::Any::new(value).expect("value failed to serialize"),
            );
        self
    }
    ///User id (if available).
    #[inline]
    pub fn uid<T>(mut self, uid: T) -> Self
    where
        T: Into<Option<super::UserId>>,
    {
        self.uid = uid.into();
        self
    }
    ///Session id (if available)
    #[inline]
    pub fn sid<T>(mut self, sid: T) -> Self
    where
        T: Into<Option<super::SessionId>>,
    {
        self.sid = sid.into();
        self
    }
    ///API token id (if available)
    #[inline]
    pub fn token_id<T>(mut self, token_id: T) -> Self
    where
        T: Into<Option<super::TokenId>>,
    {
        self.token_id = token_id.into();
        self
    }
    ///Zipkin trace id (if available)
    #[inline]
    pub fn trace_id<T>(mut self, trace_id: T) -> Self
    where
        T: Into<Option<super::TraceId>>,
    {
        self.trace_id = trace_id.into();
        self
    }
    ///Language-specific stack trace. Content is knowably safe. Renderers should substitute named placeholders ({name}, for name as a key) with keyed value from unsafeParams and leave non-matching keys as the original placeholder text.
    #[inline]
    pub fn stacktrace<T>(mut self, stacktrace: T) -> Self
    where
        T: Into<Option<String>>,
    {
        self.stacktrace = stacktrace.into();
        self
    }
    ///Unredacted parameters
    #[inline]
    pub fn unsafe_params<T>(mut self, unsafe_params: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.unsafe_params = unsafe_params.into_iter().collect();
        self
    }
    ///Unredacted parameters
    #[inline]
    pub fn extend_unsafe_params<T>(mut self, unsafe_params: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.unsafe_params.extend(unsafe_params);
        self
    }
    ///Unredacted parameters
    #[inline]
    pub fn insert_unsafe_params<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: conjure_object::serde::Serialize,
    {
        self.unsafe_params
            .insert(
                key.into(),
                conjure_object::Any::new(value).expect("value failed to serialize"),
            );
        self
    }
    ///Additional dimensions that describe the instance of the log event
    #[inline]
    pub fn tags<T>(mut self, tags: T) -> Self
    where
        T: IntoIterator<Item = (String, String)>,
    {
        self.tags = tags.into_iter().collect();
        self
    }
    ///Additional dimensions that describe the instance of the log event
    #[inline]
    pub fn extend_tags<T>(mut self, tags: T) -> Self
    where
        T: IntoIterator<Item = (String, String)>,
    {
        self.tags.extend(tags);
        self
    }
    ///Additional dimensions that describe the instance of the log event
    #[inline]
    pub fn insert_tags<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.tags.insert(key.into(), value.into());
        self
    }
    /// Consumes the builder, constructing a new instance of the type.
    #[inline]
    pub fn build(self) -> ServiceLogV1 {
        ServiceLogV1 {
            type_: self.type_,
            level: self.level,
            time: self.time,
            origin: self.origin,
            thread: self.thread,
            message: self.message,
            safe: self.safe,
            params: self.params,
            uid: self.uid,
            sid: self.sid,
            token_id: self.token_id,
            trace_id: self.trace_id,
            stacktrace: self.stacktrace,
            unsafe_params: self.unsafe_params,
            tags: self.tags,
        }
    }
}
impl ser::Serialize for ServiceLogV1 {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut size = 4usize;
        let skip_origin = self.origin.is_none();
        if !skip_origin {
            size += 1;
        }
        let skip_thread = self.thread.is_none();
        if !skip_thread {
            size += 1;
        }
        let skip_safe = self.safe.is_none();
        if !skip_safe {
            size += 1;
        }
        let skip_params = self.params.is_empty();
        if !skip_params {
            size += 1;
        }
        let skip_uid = self.uid.is_none();
        if !skip_uid {
            size += 1;
        }
        let skip_sid = self.sid.is_none();
        if !skip_sid {
            size += 1;
        }
        let skip_token_id = self.token_id.is_none();
        if !skip_token_id {
            size += 1;
        }
        let skip_trace_id = self.trace_id.is_none();
        if !skip_trace_id {
            size += 1;
        }
        let skip_stacktrace = self.stacktrace.is_none();
        if !skip_stacktrace {
            size += 1;
        }
        let skip_unsafe_params = self.unsafe_params.is_empty();
        if !skip_unsafe_params {
            size += 1;
        }
        let skip_tags = self.tags.is_empty();
        if !skip_tags {
            size += 1;
        }
        let mut s = s.serialize_struct("ServiceLogV1", size)?;
        s.serialize_field("type", &self.type_)?;
        s.serialize_field("level", &self.level)?;
        s.serialize_field("time", &self.time)?;
        if skip_origin {
            s.skip_field("origin")?;
        } else {
            s.serialize_field("origin", &self.origin)?;
        }
        if skip_thread {
            s.skip_field("thread")?;
        } else {
            s.serialize_field("thread", &self.thread)?;
        }
        s.serialize_field("message", &self.message)?;
        if skip_safe {
            s.skip_field("safe")?;
        } else {
            s.serialize_field("safe", &self.safe)?;
        }
        if skip_params {
            s.skip_field("params")?;
        } else {
            s.serialize_field("params", &self.params)?;
        }
        if skip_uid {
            s.skip_field("uid")?;
        } else {
            s.serialize_field("uid", &self.uid)?;
        }
        if skip_sid {
            s.skip_field("sid")?;
        } else {
            s.serialize_field("sid", &self.sid)?;
        }
        if skip_token_id {
            s.skip_field("tokenId")?;
        } else {
            s.serialize_field("tokenId", &self.token_id)?;
        }
        if skip_trace_id {
            s.skip_field("traceId")?;
        } else {
            s.serialize_field("traceId", &self.trace_id)?;
        }
        if skip_stacktrace {
            s.skip_field("stacktrace")?;
        } else {
            s.serialize_field("stacktrace", &self.stacktrace)?;
        }
        if skip_unsafe_params {
            s.skip_field("unsafeParams")?;
        } else {
            s.serialize_field("unsafeParams", &self.unsafe_params)?;
        }
        if skip_tags {
            s.skip_field("tags")?;
        } else {
            s.serialize_field("tags", &self.tags)?;
        }
        s.end()
    }
}
impl<'de> de::Deserialize<'de> for ServiceLogV1 {
    fn deserialize<D>(d: D) -> Result<ServiceLogV1, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_struct(
            "ServiceLogV1",
            &[
                "type",
                "level",
                "time",
                "origin",
                "thread",
                "message",
                "safe",
                "params",
                "uid",
                "sid",
                "tokenId",
                "traceId",
                "stacktrace",
                "unsafeParams",
                "tags",
            ],
            Visitor_,
        )
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = ServiceLogV1;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("map")
    }
    fn visit_map<A>(self, mut map_: A) -> Result<ServiceLogV1, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut type_ = None;
        let mut level = None;
        let mut time = None;
        let mut origin = None;
        let mut thread = None;
        let mut message = None;
        let mut safe = None;
        let mut params = None;
        let mut uid = None;
        let mut sid = None;
        let mut token_id = None;
        let mut trace_id = None;
        let mut stacktrace = None;
        let mut unsafe_params = None;
        let mut tags = None;
        while let Some(field_) = map_.next_key()? {
            match field_ {
                Field_::Type => type_ = Some(map_.next_value()?),
                Field_::Level => level = Some(map_.next_value()?),
                Field_::Time => time = Some(map_.next_value()?),
                Field_::Origin => origin = Some(map_.next_value()?),
                Field_::Thread => thread = Some(map_.next_value()?),
                Field_::Message => message = Some(map_.next_value()?),
                Field_::Safe => safe = Some(map_.next_value()?),
                Field_::Params => params = Some(map_.next_value()?),
                Field_::Uid => uid = Some(map_.next_value()?),
                Field_::Sid => sid = Some(map_.next_value()?),
                Field_::TokenId => token_id = Some(map_.next_value()?),
                Field_::TraceId => trace_id = Some(map_.next_value()?),
                Field_::Stacktrace => stacktrace = Some(map_.next_value()?),
                Field_::UnsafeParams => unsafe_params = Some(map_.next_value()?),
                Field_::Tags => tags = Some(map_.next_value()?),
                Field_::Unknown_ => {
                    map_.next_value::<de::IgnoredAny>()?;
                }
            }
        }
        let type_ = match type_ {
            Some(v) => v,
            None => return Err(de::Error::missing_field("type")),
        };
        let level = match level {
            Some(v) => v,
            None => return Err(de::Error::missing_field("level")),
        };
        let time = match time {
            Some(v) => v,
            None => return Err(de::Error::missing_field("time")),
        };
        let origin = match origin {
            Some(v) => v,
            None => Default::default(),
        };
        let thread = match thread {
            Some(v) => v,
            None => Default::default(),
        };
        let message = match message {
            Some(v) => v,
            None => return Err(de::Error::missing_field("message")),
        };
        let safe = match safe {
            Some(v) => v,
            None => Default::default(),
        };
        let params = match params {
            Some(v) => v,
            None => Default::default(),
        };
        let uid = match uid {
            Some(v) => v,
            None => Default::default(),
        };
        let sid = match sid {
            Some(v) => v,
            None => Default::default(),
        };
        let token_id = match token_id {
            Some(v) => v,
            None => Default::default(),
        };
        let trace_id = match trace_id {
            Some(v) => v,
            None => Default::default(),
        };
        let stacktrace = match stacktrace {
            Some(v) => v,
            None => Default::default(),
        };
        let unsafe_params = match unsafe_params {
            Some(v) => v,
            None => Default::default(),
        };
        let tags = match tags {
            Some(v) => v,
            None => Default::default(),
        };
        Ok(ServiceLogV1 {
            type_,
            level,
            time,
            origin,
            thread,
            message,
            safe,
            params,
            uid,
            sid,
            token_id,
            trace_id,
            stacktrace,
            unsafe_params,
            tags,
        })
    }
}
enum Field_ {
    Type,
    Level,
    Time,
    Origin,
    Thread,
    Message,
    Safe,
    Params,
    Uid,
    Sid,
    TokenId,
    TraceId,
    Stacktrace,
    UnsafeParams,
    Tags,
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
            "level" => Field_::Level,
            "time" => Field_::Time,
            "origin" => Field_::Origin,
            "thread" => Field_::Thread,
            "message" => Field_::Message,
            "safe" => Field_::Safe,
            "params" => Field_::Params,
            "uid" => Field_::Uid,
            "sid" => Field_::Sid,
            "tokenId" => Field_::TokenId,
            "traceId" => Field_::TraceId,
            "stacktrace" => Field_::Stacktrace,
            "unsafeParams" => Field_::UnsafeParams,
            "tags" => Field_::Tags,
            _ => Field_::Unknown_,
        };
        Ok(v)
    }
}
