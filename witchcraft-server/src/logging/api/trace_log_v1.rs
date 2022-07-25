use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
///Definition of the trace.1 format.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TraceLogV1 {
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
    uid: Option<super::UserId>,
    sid: Option<super::SessionId>,
    token_id: Option<super::TokenId>,
    unsafe_params: std::collections::BTreeMap<String, conjure_object::Any>,
    span: Box<super::Span>,
}
impl TraceLogV1 {
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
    #[inline]
    pub fn uid(&self) -> Option<&super::UserId> {
        self.uid.as_ref().map(|o| &*o)
    }
    #[inline]
    pub fn sid(&self) -> Option<&super::SessionId> {
        self.sid.as_ref().map(|o| &*o)
    }
    #[inline]
    pub fn token_id(&self) -> Option<&super::TokenId> {
        self.token_id.as_ref().map(|o| &*o)
    }
    #[inline]
    pub fn unsafe_params(
        &self,
    ) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.unsafe_params
    }
    #[inline]
    pub fn span(&self) -> &super::Span {
        &*self.span
    }
}
impl Default for BuilderStage0 {
    #[inline]
    fn default() -> Self {
        BuilderStage0 {}
    }
}
impl From<TraceLogV1> for BuilderStage3 {
    #[inline]
    fn from(value: TraceLogV1) -> Self {
        BuilderStage3 {
            type_: value.type_,
            time: value.time,
            uid: value.uid,
            sid: value.sid,
            token_id: value.token_id,
            unsafe_params: value.unsafe_params,
            span: value.span,
        }
    }
}
///The stage 0 builder for the [`TraceLogV1`] type
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
///The stage 1 builder for the [`TraceLogV1`] type
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
///The stage 2 builder for the [`TraceLogV1`] type
#[derive(Debug, Clone)]
pub struct BuilderStage2 {
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
}
impl BuilderStage2 {
    #[inline]
    pub fn span(self, span: super::Span) -> BuilderStage3 {
        BuilderStage3 {
            type_: self.type_,
            time: self.time,
            span: Box::new(span),
            uid: Default::default(),
            sid: Default::default(),
            token_id: Default::default(),
            unsafe_params: Default::default(),
        }
    }
}
///The stage 3 builder for the [`TraceLogV1`] type
#[derive(Debug, Clone)]
pub struct BuilderStage3 {
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
    span: Box<super::Span>,
    uid: Option<super::UserId>,
    sid: Option<super::SessionId>,
    token_id: Option<super::TokenId>,
    unsafe_params: std::collections::BTreeMap<String, conjure_object::Any>,
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
    #[inline]
    pub fn span(mut self, span: super::Span) -> Self {
        self.span = Box::new(span);
        self
    }
    #[inline]
    pub fn uid<T>(mut self, uid: T) -> Self
    where
        T: Into<Option<super::UserId>>,
    {
        self.uid = uid.into();
        self
    }
    #[inline]
    pub fn sid<T>(mut self, sid: T) -> Self
    where
        T: Into<Option<super::SessionId>>,
    {
        self.sid = sid.into();
        self
    }
    #[inline]
    pub fn token_id<T>(mut self, token_id: T) -> Self
    where
        T: Into<Option<super::TokenId>>,
    {
        self.token_id = token_id.into();
        self
    }
    #[inline]
    pub fn unsafe_params<T>(mut self, unsafe_params: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.unsafe_params = unsafe_params.into_iter().collect();
        self
    }
    #[inline]
    pub fn extend_unsafe_params<T>(mut self, unsafe_params: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.unsafe_params.extend(unsafe_params);
        self
    }
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
    /// Consumes the builder, constructing a new instance of the type.
    #[inline]
    pub fn build(self) -> TraceLogV1 {
        TraceLogV1 {
            type_: self.type_,
            time: self.time,
            uid: self.uid,
            sid: self.sid,
            token_id: self.token_id,
            unsafe_params: self.unsafe_params,
            span: self.span,
        }
    }
}
impl ser::Serialize for TraceLogV1 {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut size = 3usize;
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
        let mut s = s.serialize_struct("TraceLogV1", size)?;
        s.serialize_field("type", &self.type_)?;
        s.serialize_field("time", &self.time)?;
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
        s.serialize_field("span", &self.span)?;
        s.end()
    }
}
impl<'de> de::Deserialize<'de> for TraceLogV1 {
    fn deserialize<D>(d: D) -> Result<TraceLogV1, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_struct(
            "TraceLogV1",
            &["type", "time", "uid", "sid", "tokenId", "unsafeParams", "span"],
            Visitor_,
        )
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = TraceLogV1;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("map")
    }
    fn visit_map<A>(self, mut map_: A) -> Result<TraceLogV1, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut type_ = None;
        let mut time = None;
        let mut uid = None;
        let mut sid = None;
        let mut token_id = None;
        let mut unsafe_params = None;
        let mut span = None;
        while let Some(field_) = map_.next_key()? {
            match field_ {
                Field_::Type => type_ = Some(map_.next_value()?),
                Field_::Time => time = Some(map_.next_value()?),
                Field_::Uid => uid = Some(map_.next_value()?),
                Field_::Sid => sid = Some(map_.next_value()?),
                Field_::TokenId => token_id = Some(map_.next_value()?),
                Field_::UnsafeParams => unsafe_params = Some(map_.next_value()?),
                Field_::Span => span = Some(map_.next_value()?),
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
        let span = match span {
            Some(v) => v,
            None => return Err(de::Error::missing_field("span")),
        };
        Ok(TraceLogV1 {
            type_,
            time,
            uid,
            sid,
            token_id,
            unsafe_params,
            span,
        })
    }
}
enum Field_ {
    Type,
    Time,
    Uid,
    Sid,
    TokenId,
    UnsafeParams,
    Span,
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
            "uid" => Field_::Uid,
            "sid" => Field_::Sid,
            "tokenId" => Field_::TokenId,
            "unsafeParams" => Field_::UnsafeParams,
            "span" => Field_::Span,
            _ => Field_::Unknown_,
        };
        Ok(v)
    }
}
