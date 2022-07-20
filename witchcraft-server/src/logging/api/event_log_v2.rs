use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
///Definition of the event.2 format.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventLogV2 {
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
    event_name: String,
    values: std::collections::BTreeMap<String, conjure_object::Any>,
    uid: Option<super::UserId>,
    sid: Option<super::SessionId>,
    token_id: Option<super::TokenId>,
    trace_id: Option<super::TraceId>,
    unsafe_params: std::collections::BTreeMap<String, conjure_object::Any>,
    tags: std::collections::BTreeMap<String, String>,
}
impl EventLogV2 {
    /// Returns a new builder.
    #[inline]
    pub fn builder() -> BuilderStage0 {
        Default::default()
    }
    #[inline]
    pub fn type_(&self) -> &str {
        &*self.type_
    }
    #[inline]
    pub fn time(&self) -> conjure_object::DateTime<conjure_object::Utc> {
        self.time
    }
    ///Dot-delimited name of event, e.g. `com.foundry.compass.api.Compass.http.ping.failures`
    #[inline]
    pub fn event_name(&self) -> &str {
        &*self.event_name
    }
    ///Observations, measurements and context associated with the event
    #[inline]
    pub fn values(&self) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.values
    }
    ///User id (if available)
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
    ///Unsafe metadata describing the event
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
impl From<EventLogV2> for BuilderStage3 {
    #[inline]
    fn from(value: EventLogV2) -> Self {
        BuilderStage3 {
            type_: value.type_,
            time: value.time,
            event_name: value.event_name,
            values: value.values,
            uid: value.uid,
            sid: value.sid,
            token_id: value.token_id,
            trace_id: value.trace_id,
            unsafe_params: value.unsafe_params,
            tags: value.tags,
        }
    }
}
///The stage 0 builder for the [`EventLogV2`] type
#[derive(Debug, Clone)]
pub struct BuilderStage0 {}
impl BuilderStage0 {
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
///The stage 1 builder for the [`EventLogV2`] type
#[derive(Debug, Clone)]
pub struct BuilderStage1 {
    type_: String,
}
impl BuilderStage1 {
    #[inline]
    pub fn time(
        self,
        time: conjure_object::DateTime<conjure_object::Utc>,
    ) -> BuilderStage2 {
        BuilderStage2 {
            type_: self.type_,
            time: time,
        }
    }
}
///The stage 2 builder for the [`EventLogV2`] type
#[derive(Debug, Clone)]
pub struct BuilderStage2 {
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
}
impl BuilderStage2 {
    ///Dot-delimited name of event, e.g. `com.foundry.compass.api.Compass.http.ping.failures`
    #[inline]
    pub fn event_name<T>(self, event_name: T) -> BuilderStage3
    where
        T: Into<String>,
    {
        BuilderStage3 {
            type_: self.type_,
            time: self.time,
            event_name: event_name.into(),
            values: Default::default(),
            uid: Default::default(),
            sid: Default::default(),
            token_id: Default::default(),
            trace_id: Default::default(),
            unsafe_params: Default::default(),
            tags: Default::default(),
        }
    }
}
///The stage 3 builder for the [`EventLogV2`] type
#[derive(Debug, Clone)]
pub struct BuilderStage3 {
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
    event_name: String,
    values: std::collections::BTreeMap<String, conjure_object::Any>,
    uid: Option<super::UserId>,
    sid: Option<super::SessionId>,
    token_id: Option<super::TokenId>,
    trace_id: Option<super::TraceId>,
    unsafe_params: std::collections::BTreeMap<String, conjure_object::Any>,
    tags: std::collections::BTreeMap<String, String>,
}
impl BuilderStage3 {
    #[inline]
    pub fn type_<T>(mut self, type_: T) -> Self
    where
        T: Into<String>,
    {
        self.type_ = type_.into();
        self
    }
    #[inline]
    pub fn time(mut self, time: conjure_object::DateTime<conjure_object::Utc>) -> Self {
        self.time = time;
        self
    }
    ///Dot-delimited name of event, e.g. `com.foundry.compass.api.Compass.http.ping.failures`
    #[inline]
    pub fn event_name<T>(mut self, event_name: T) -> Self
    where
        T: Into<String>,
    {
        self.event_name = event_name.into();
        self
    }
    ///Observations, measurements and context associated with the event
    #[inline]
    pub fn values<T>(mut self, values: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.values = values.into_iter().collect();
        self
    }
    ///Observations, measurements and context associated with the event
    #[inline]
    pub fn extend_values<T>(mut self, values: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.values.extend(values);
        self
    }
    ///Observations, measurements and context associated with the event
    #[inline]
    pub fn insert_values<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: conjure_object::serde::Serialize,
    {
        self.values
            .insert(
                key.into(),
                conjure_object::Any::new(value).expect("value failed to serialize"),
            );
        self
    }
    ///User id (if available)
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
    ///Unsafe metadata describing the event
    #[inline]
    pub fn unsafe_params<T>(mut self, unsafe_params: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.unsafe_params = unsafe_params.into_iter().collect();
        self
    }
    ///Unsafe metadata describing the event
    #[inline]
    pub fn extend_unsafe_params<T>(mut self, unsafe_params: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.unsafe_params.extend(unsafe_params);
        self
    }
    ///Unsafe metadata describing the event
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
    pub fn build(self) -> EventLogV2 {
        EventLogV2 {
            type_: self.type_,
            time: self.time,
            event_name: self.event_name,
            values: self.values,
            uid: self.uid,
            sid: self.sid,
            token_id: self.token_id,
            trace_id: self.trace_id,
            unsafe_params: self.unsafe_params,
            tags: self.tags,
        }
    }
}
impl ser::Serialize for EventLogV2 {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut size = 3usize;
        let skip_values = self.values.is_empty();
        if !skip_values {
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
        let skip_unsafe_params = self.unsafe_params.is_empty();
        if !skip_unsafe_params {
            size += 1;
        }
        let skip_tags = self.tags.is_empty();
        if !skip_tags {
            size += 1;
        }
        let mut s = s.serialize_struct("EventLogV2", size)?;
        s.serialize_field("type", &self.type_)?;
        s.serialize_field("time", &self.time)?;
        s.serialize_field("eventName", &self.event_name)?;
        if skip_values {
            s.skip_field("values")?;
        } else {
            s.serialize_field("values", &self.values)?;
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
impl<'de> de::Deserialize<'de> for EventLogV2 {
    fn deserialize<D>(d: D) -> Result<EventLogV2, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_struct(
            "EventLogV2",
            &[
                "type",
                "time",
                "eventName",
                "values",
                "uid",
                "sid",
                "tokenId",
                "traceId",
                "unsafeParams",
                "tags",
            ],
            Visitor_,
        )
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = EventLogV2;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("map")
    }
    fn visit_map<A>(self, mut map_: A) -> Result<EventLogV2, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut type_ = None;
        let mut time = None;
        let mut event_name = None;
        let mut values = None;
        let mut uid = None;
        let mut sid = None;
        let mut token_id = None;
        let mut trace_id = None;
        let mut unsafe_params = None;
        let mut tags = None;
        while let Some(field_) = map_.next_key()? {
            match field_ {
                Field_::Type => type_ = Some(map_.next_value()?),
                Field_::Time => time = Some(map_.next_value()?),
                Field_::EventName => event_name = Some(map_.next_value()?),
                Field_::Values => values = Some(map_.next_value()?),
                Field_::Uid => uid = Some(map_.next_value()?),
                Field_::Sid => sid = Some(map_.next_value()?),
                Field_::TokenId => token_id = Some(map_.next_value()?),
                Field_::TraceId => trace_id = Some(map_.next_value()?),
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
        let time = match time {
            Some(v) => v,
            None => return Err(de::Error::missing_field("time")),
        };
        let event_name = match event_name {
            Some(v) => v,
            None => return Err(de::Error::missing_field("eventName")),
        };
        let values = match values {
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
        let unsafe_params = match unsafe_params {
            Some(v) => v,
            None => Default::default(),
        };
        let tags = match tags {
            Some(v) => v,
            None => Default::default(),
        };
        Ok(EventLogV2 {
            type_,
            time,
            event_name,
            values,
            uid,
            sid,
            token_id,
            trace_id,
            unsafe_params,
            tags,
        })
    }
}
enum Field_ {
    Type,
    Time,
    EventName,
    Values,
    Uid,
    Sid,
    TokenId,
    TraceId,
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
            "time" => Field_::Time,
            "eventName" => Field_::EventName,
            "values" => Field_::Values,
            "uid" => Field_::Uid,
            "sid" => Field_::Sid,
            "tokenId" => Field_::TokenId,
            "traceId" => Field_::TraceId,
            "unsafeParams" => Field_::UnsafeParams,
            "tags" => Field_::Tags,
            _ => Field_::Unknown_,
        };
        Ok(v)
    }
}
