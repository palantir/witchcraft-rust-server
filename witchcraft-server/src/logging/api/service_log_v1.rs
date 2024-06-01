use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
///Definition of the service.1 format.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[conjure_object::private::staged_builder::staged_builder]
#[builder(crate = conjure_object::private::staged_builder, update, inline)]
pub struct ServiceLogV1 {
    #[builder(into)]
    type_: String,
    level: super::LogLevel,
    time: conjure_object::DateTime<conjure_object::Utc>,
    #[builder(default, into)]
    origin: Option<String>,
    #[builder(default, into)]
    thread: Option<String>,
    #[builder(into)]
    message: String,
    #[builder(default, into)]
    safe: Option<bool>,
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
    #[builder(default, into)]
    uid: Option<super::UserId>,
    #[builder(default, into)]
    sid: Option<super::SessionId>,
    #[builder(default, into)]
    token_id: Option<super::TokenId>,
    #[builder(default, into)]
    org_id: Option<super::OrganizationId>,
    #[builder(default, into)]
    trace_id: Option<super::TraceId>,
    #[builder(default, into)]
    stacktrace: Option<String>,
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
    unsafe_params: std::collections::BTreeMap<String, conjure_object::Any>,
    #[builder(default, map(key(type = String, into), value(type = String, into)))]
    tags: std::collections::BTreeMap<String, String>,
}
impl ServiceLogV1 {
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
    ///Organization id (if available)
    #[inline]
    pub fn org_id(&self) -> Option<&super::OrganizationId> {
        self.org_id.as_ref().map(|o| &*o)
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
        let skip_org_id = self.org_id.is_none();
        if !skip_org_id {
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
        if skip_org_id {
            s.skip_field("orgId")?;
        } else {
            s.serialize_field("orgId", &self.org_id)?;
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
                "orgId",
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
        let mut org_id = None;
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
                Field_::OrgId => org_id = Some(map_.next_value()?),
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
        let org_id = match org_id {
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
            org_id,
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
    OrgId,
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
            "orgId" => Field_::OrgId,
            "traceId" => Field_::TraceId,
            "stacktrace" => Field_::Stacktrace,
            "unsafeParams" => Field_::UnsafeParams,
            "tags" => Field_::Tags,
            _ => Field_::Unknown_,
        };
        Ok(v)
    }
}
