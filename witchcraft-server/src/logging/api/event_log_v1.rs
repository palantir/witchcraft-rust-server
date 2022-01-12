use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use conjure_object::serde::{de, ser};
use std::fmt;
#[doc = "Definition of the event.1 format."]
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct EventLogV1 {
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
    event_name: String,
    event_type: String,
    values: std::collections::BTreeMap<String, conjure_object::Any>,
    uid: Option<super::UserId>,
    sid: Option<super::SessionId>,
    token_id: Option<super::TokenId>,
    unsafe_params: std::collections::BTreeMap<String, conjure_object::Any>,
}
impl EventLogV1 {
    #[doc = r" Returns a new builder."]
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
    #[doc = "Dot-delimited name of event, e.g. `com.foundry.compass.api.Compass.http.ping.failures`"]
    #[inline]
    pub fn event_name(&self) -> &str {
        &*self.event_name
    }
    #[doc = "Type of event being represented, e.g. `gauge`, `histogram`, `counter`"]
    #[inline]
    pub fn event_type(&self) -> &str {
        &*self.event_type
    }
    #[doc = "Observations, measurements and context associated with the event"]
    #[inline]
    pub fn values(&self) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.values
    }
    #[doc = "User id (if available)"]
    #[inline]
    pub fn uid(&self) -> Option<&super::UserId> {
        self.uid.as_ref().map(|o| &*o)
    }
    #[doc = "Session id (if available)"]
    #[inline]
    pub fn sid(&self) -> Option<&super::SessionId> {
        self.sid.as_ref().map(|o| &*o)
    }
    #[doc = "API token id (if available)"]
    #[inline]
    pub fn token_id(&self) -> Option<&super::TokenId> {
        self.token_id.as_ref().map(|o| &*o)
    }
    #[doc = "Unsafe metadata describing the event"]
    #[inline]
    pub fn unsafe_params(&self) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.unsafe_params
    }
}
impl Default for BuilderStage0 {
    #[inline]
    fn default() -> Self {
        BuilderStage0 {}
    }
}
impl From<EventLogV1> for BuilderStage4 {
    #[inline]
    fn from(value: EventLogV1) -> Self {
        BuilderStage4 {
            type_: value.type_,
            time: value.time,
            event_name: value.event_name,
            event_type: value.event_type,
            values: value.values,
            uid: value.uid,
            sid: value.sid,
            token_id: value.token_id,
            unsafe_params: value.unsafe_params,
        }
    }
}
#[doc = "The stage 0 builder for the [`EventLogV1`] type"]
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
#[doc = "The stage 1 builder for the [`EventLogV1`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage1 {
    type_: String,
}
impl BuilderStage1 {
    #[inline]
    pub fn time(self, time: conjure_object::DateTime<conjure_object::Utc>) -> BuilderStage2 {
        BuilderStage2 {
            type_: self.type_,
            time: time,
        }
    }
}
#[doc = "The stage 2 builder for the [`EventLogV1`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage2 {
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
}
impl BuilderStage2 {
    #[doc = "Dot-delimited name of event, e.g. `com.foundry.compass.api.Compass.http.ping.failures`"]
    #[inline]
    pub fn event_name<T>(self, event_name: T) -> BuilderStage3
    where
        T: Into<String>,
    {
        BuilderStage3 {
            type_: self.type_,
            time: self.time,
            event_name: event_name.into(),
        }
    }
}
#[doc = "The stage 3 builder for the [`EventLogV1`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage3 {
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
    event_name: String,
}
impl BuilderStage3 {
    #[doc = "Type of event being represented, e.g. `gauge`, `histogram`, `counter`"]
    #[inline]
    pub fn event_type<T>(self, event_type: T) -> BuilderStage4
    where
        T: Into<String>,
    {
        BuilderStage4 {
            type_: self.type_,
            time: self.time,
            event_name: self.event_name,
            event_type: event_type.into(),
            values: Default::default(),
            uid: Default::default(),
            sid: Default::default(),
            token_id: Default::default(),
            unsafe_params: Default::default(),
        }
    }
}
#[doc = "The stage 4 builder for the [`EventLogV1`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage4 {
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
    event_name: String,
    event_type: String,
    values: std::collections::BTreeMap<String, conjure_object::Any>,
    uid: Option<super::UserId>,
    sid: Option<super::SessionId>,
    token_id: Option<super::TokenId>,
    unsafe_params: std::collections::BTreeMap<String, conjure_object::Any>,
}
impl BuilderStage4 {
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
    #[doc = "Dot-delimited name of event, e.g. `com.foundry.compass.api.Compass.http.ping.failures`"]
    #[inline]
    pub fn event_name<T>(mut self, event_name: T) -> Self
    where
        T: Into<String>,
    {
        self.event_name = event_name.into();
        self
    }
    #[doc = "Type of event being represented, e.g. `gauge`, `histogram`, `counter`"]
    #[inline]
    pub fn event_type<T>(mut self, event_type: T) -> Self
    where
        T: Into<String>,
    {
        self.event_type = event_type.into();
        self
    }
    #[doc = "Observations, measurements and context associated with the event"]
    #[inline]
    pub fn values<T>(mut self, values: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.values = values.into_iter().collect();
        self
    }
    #[doc = "Observations, measurements and context associated with the event"]
    #[inline]
    pub fn extend_values<T>(mut self, values: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.values.extend(values);
        self
    }
    #[doc = "Observations, measurements and context associated with the event"]
    #[inline]
    pub fn insert_values<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: conjure_object::serde::Serialize,
    {
        self.values.insert(
            key.into(),
            conjure_object::Any::new(value).expect("value failed to serialize"),
        );
        self
    }
    #[doc = "User id (if available)"]
    #[inline]
    pub fn uid<T>(mut self, uid: T) -> Self
    where
        T: Into<Option<super::UserId>>,
    {
        self.uid = uid.into();
        self
    }
    #[doc = "Session id (if available)"]
    #[inline]
    pub fn sid<T>(mut self, sid: T) -> Self
    where
        T: Into<Option<super::SessionId>>,
    {
        self.sid = sid.into();
        self
    }
    #[doc = "API token id (if available)"]
    #[inline]
    pub fn token_id<T>(mut self, token_id: T) -> Self
    where
        T: Into<Option<super::TokenId>>,
    {
        self.token_id = token_id.into();
        self
    }
    #[doc = "Unsafe metadata describing the event"]
    #[inline]
    pub fn unsafe_params<T>(mut self, unsafe_params: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.unsafe_params = unsafe_params.into_iter().collect();
        self
    }
    #[doc = "Unsafe metadata describing the event"]
    #[inline]
    pub fn extend_unsafe_params<T>(mut self, unsafe_params: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.unsafe_params.extend(unsafe_params);
        self
    }
    #[doc = "Unsafe metadata describing the event"]
    #[inline]
    pub fn insert_unsafe_params<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: conjure_object::serde::Serialize,
    {
        self.unsafe_params.insert(
            key.into(),
            conjure_object::Any::new(value).expect("value failed to serialize"),
        );
        self
    }
    #[doc = r" Consumes the builder, constructing a new instance of the type."]
    #[inline]
    pub fn build(self) -> EventLogV1 {
        EventLogV1 {
            type_: self.type_,
            time: self.time,
            event_name: self.event_name,
            event_type: self.event_type,
            values: self.values,
            uid: self.uid,
            sid: self.sid,
            token_id: self.token_id,
            unsafe_params: self.unsafe_params,
        }
    }
}
impl ser::Serialize for EventLogV1 {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut size = 4usize;
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
        let skip_unsafe_params = self.unsafe_params.is_empty();
        if !skip_unsafe_params {
            size += 1;
        }
        let mut s = s.serialize_struct("EventLogV1", size)?;
        s.serialize_field("type", &self.type_)?;
        s.serialize_field("time", &self.time)?;
        s.serialize_field("eventName", &self.event_name)?;
        s.serialize_field("eventType", &self.event_type)?;
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
        if skip_unsafe_params {
            s.skip_field("unsafeParams")?;
        } else {
            s.serialize_field("unsafeParams", &self.unsafe_params)?;
        }
        s.end()
    }
}
impl<'de> de::Deserialize<'de> for EventLogV1 {
    fn deserialize<D>(d: D) -> Result<EventLogV1, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_struct(
            "EventLogV1",
            &[
                "type",
                "time",
                "eventName",
                "eventType",
                "values",
                "uid",
                "sid",
                "tokenId",
                "unsafeParams",
            ],
            Visitor_,
        )
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = EventLogV1;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("map")
    }
    fn visit_map<A>(self, mut map_: A) -> Result<EventLogV1, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut type_ = None;
        let mut time = None;
        let mut event_name = None;
        let mut event_type = None;
        let mut values = None;
        let mut uid = None;
        let mut sid = None;
        let mut token_id = None;
        let mut unsafe_params = None;
        while let Some(field_) = map_.next_key()? {
            match field_ {
                Field_::Type => type_ = Some(map_.next_value()?),
                Field_::Time => time = Some(map_.next_value()?),
                Field_::EventName => event_name = Some(map_.next_value()?),
                Field_::EventType => event_type = Some(map_.next_value()?),
                Field_::Values => values = Some(map_.next_value()?),
                Field_::Uid => uid = Some(map_.next_value()?),
                Field_::Sid => sid = Some(map_.next_value()?),
                Field_::TokenId => token_id = Some(map_.next_value()?),
                Field_::UnsafeParams => unsafe_params = Some(map_.next_value()?),
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
        let event_type = match event_type {
            Some(v) => v,
            None => return Err(de::Error::missing_field("eventType")),
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
        let unsafe_params = match unsafe_params {
            Some(v) => v,
            None => Default::default(),
        };
        Ok(EventLogV1 {
            type_,
            time,
            event_name,
            event_type,
            values,
            uid,
            sid,
            token_id,
            unsafe_params,
        })
    }
}
enum Field_ {
    Type,
    Time,
    EventName,
    EventType,
    Values,
    Uid,
    Sid,
    TokenId,
    UnsafeParams,
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
            "eventType" => Field_::EventType,
            "values" => Field_::Values,
            "uid" => Field_::Uid,
            "sid" => Field_::Sid,
            "tokenId" => Field_::TokenId,
            "unsafeParams" => Field_::UnsafeParams,
            _ => Field_::Unknown_,
        };
        Ok(v)
    }
}
