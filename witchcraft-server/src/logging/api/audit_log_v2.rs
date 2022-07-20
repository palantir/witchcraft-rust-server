use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use conjure_object::serde::{de, ser};
use std::fmt;
///Definition of the audit.2 format.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AuditLogV2 {
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
    uid: Option<super::UserId>,
    sid: Option<super::SessionId>,
    token_id: Option<super::TokenId>,
    trace_id: Option<super::TraceId>,
    other_uids: Vec<super::UserId>,
    origin: Option<String>,
    name: String,
    result: super::AuditResult,
    request_params: std::collections::BTreeMap<String, conjure_object::Any>,
    result_params: std::collections::BTreeMap<String, conjure_object::Any>,
}
impl AuditLogV2 {
    /// Returns a new builder.
    #[inline]
    pub fn builder() -> BuilderStage0 {
        Default::default()
    }
    ///"audit.2"
    #[inline]
    pub fn type_(&self) -> &str {
        &*self.type_
    }
    #[inline]
    pub fn time(&self) -> conjure_object::DateTime<conjure_object::Utc> {
        self.time
    }
    ///User id (if available). This is the most downstream caller.
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
    ///All users upstream of the user currently taking an action. The first element in this list is the uid of the most upstream caller. This list does not include the `uid`.
    #[inline]
    pub fn other_uids(&self) -> &[super::UserId] {
        &*self.other_uids
    }
    ///Best-effort identifier of the originating machine, e.g. an IP address, a Kubernetes node identifier,
    ///or similar
    #[inline]
    pub fn origin(&self) -> Option<&str> {
        self.origin.as_ref().map(|o| &**o)
    }
    ///Name of the audit event, e.g. PUT_FILE
    #[inline]
    pub fn name(&self) -> &str {
        &*self.name
    }
    ///Indicates whether the request was successful or the type of failure, e.g. ERROR or UNAUTHORIZED
    #[inline]
    pub fn result(&self) -> &super::AuditResult {
        &self.result
    }
    ///The parameters known at method invocation time.
    #[inline]
    pub fn request_params(&self) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.request_params
    }
    ///Information derived within a method, commonly parts of the return value.
    #[inline]
    pub fn result_params(&self) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.result_params
    }
}
impl Default for BuilderStage0 {
    #[inline]
    fn default() -> Self {
        BuilderStage0 {}
    }
}
impl From<AuditLogV2> for BuilderStage4 {
    #[inline]
    fn from(value: AuditLogV2) -> Self {
        BuilderStage4 {
            type_: value.type_,
            time: value.time,
            uid: value.uid,
            sid: value.sid,
            token_id: value.token_id,
            trace_id: value.trace_id,
            other_uids: value.other_uids,
            origin: value.origin,
            name: value.name,
            result: value.result,
            request_params: value.request_params,
            result_params: value.result_params,
        }
    }
}
///The stage 0 builder for the [`AuditLogV2`] type
#[derive(Debug, Clone)]
pub struct BuilderStage0 {}
impl BuilderStage0 {
    ///"audit.2"
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
///The stage 1 builder for the [`AuditLogV2`] type
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
///The stage 2 builder for the [`AuditLogV2`] type
#[derive(Debug, Clone)]
pub struct BuilderStage2 {
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
}
impl BuilderStage2 {
    ///Name of the audit event, e.g. PUT_FILE
    #[inline]
    pub fn name<T>(self, name: T) -> BuilderStage3
    where
        T: Into<String>,
    {
        BuilderStage3 {
            type_: self.type_,
            time: self.time,
            name: name.into(),
        }
    }
}
///The stage 3 builder for the [`AuditLogV2`] type
#[derive(Debug, Clone)]
pub struct BuilderStage3 {
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
    name: String,
}
impl BuilderStage3 {
    ///Indicates whether the request was successful or the type of failure, e.g. ERROR or UNAUTHORIZED
    #[inline]
    pub fn result(self, result: super::AuditResult) -> BuilderStage4 {
        BuilderStage4 {
            type_: self.type_,
            time: self.time,
            name: self.name,
            result: result,
            uid: Default::default(),
            sid: Default::default(),
            token_id: Default::default(),
            trace_id: Default::default(),
            other_uids: Default::default(),
            origin: Default::default(),
            request_params: Default::default(),
            result_params: Default::default(),
        }
    }
}
///The stage 4 builder for the [`AuditLogV2`] type
#[derive(Debug, Clone)]
pub struct BuilderStage4 {
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
    name: String,
    result: super::AuditResult,
    uid: Option<super::UserId>,
    sid: Option<super::SessionId>,
    token_id: Option<super::TokenId>,
    trace_id: Option<super::TraceId>,
    other_uids: Vec<super::UserId>,
    origin: Option<String>,
    request_params: std::collections::BTreeMap<String, conjure_object::Any>,
    result_params: std::collections::BTreeMap<String, conjure_object::Any>,
}
impl BuilderStage4 {
    ///"audit.2"
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
    ///Name of the audit event, e.g. PUT_FILE
    #[inline]
    pub fn name<T>(mut self, name: T) -> Self
    where
        T: Into<String>,
    {
        self.name = name.into();
        self
    }
    ///Indicates whether the request was successful or the type of failure, e.g. ERROR or UNAUTHORIZED
    #[inline]
    pub fn result(mut self, result: super::AuditResult) -> Self {
        self.result = result;
        self
    }
    ///User id (if available). This is the most downstream caller.
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
    ///All users upstream of the user currently taking an action. The first element in this list is the uid of the most upstream caller. This list does not include the `uid`.
    #[inline]
    pub fn other_uids<T>(mut self, other_uids: T) -> Self
    where
        T: IntoIterator<Item = super::UserId>,
    {
        self.other_uids = other_uids.into_iter().collect();
        self
    }
    ///All users upstream of the user currently taking an action. The first element in this list is the uid of the most upstream caller. This list does not include the `uid`.
    #[inline]
    pub fn extend_other_uids<T>(mut self, other_uids: T) -> Self
    where
        T: IntoIterator<Item = super::UserId>,
    {
        self.other_uids.extend(other_uids);
        self
    }
    ///All users upstream of the user currently taking an action. The first element in this list is the uid of the most upstream caller. This list does not include the `uid`.
    #[inline]
    pub fn push_other_uids(mut self, value: super::UserId) -> Self {
        self.other_uids.push(value);
        self
    }
    ///Best-effort identifier of the originating machine, e.g. an IP address, a Kubernetes node identifier,
    ///or similar
    #[inline]
    pub fn origin<T>(mut self, origin: T) -> Self
    where
        T: Into<Option<String>>,
    {
        self.origin = origin.into();
        self
    }
    ///The parameters known at method invocation time.
    #[inline]
    pub fn request_params<T>(mut self, request_params: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.request_params = request_params.into_iter().collect();
        self
    }
    ///The parameters known at method invocation time.
    #[inline]
    pub fn extend_request_params<T>(mut self, request_params: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.request_params.extend(request_params);
        self
    }
    ///The parameters known at method invocation time.
    #[inline]
    pub fn insert_request_params<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: conjure_object::serde::Serialize,
    {
        self.request_params.insert(
            key.into(),
            conjure_object::Any::new(value).expect("value failed to serialize"),
        );
        self
    }
    ///Information derived within a method, commonly parts of the return value.
    #[inline]
    pub fn result_params<T>(mut self, result_params: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.result_params = result_params.into_iter().collect();
        self
    }
    ///Information derived within a method, commonly parts of the return value.
    #[inline]
    pub fn extend_result_params<T>(mut self, result_params: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.result_params.extend(result_params);
        self
    }
    ///Information derived within a method, commonly parts of the return value.
    #[inline]
    pub fn insert_result_params<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: conjure_object::serde::Serialize,
    {
        self.result_params.insert(
            key.into(),
            conjure_object::Any::new(value).expect("value failed to serialize"),
        );
        self
    }
    /// Consumes the builder, constructing a new instance of the type.
    #[inline]
    pub fn build(self) -> AuditLogV2 {
        AuditLogV2 {
            type_: self.type_,
            time: self.time,
            uid: self.uid,
            sid: self.sid,
            token_id: self.token_id,
            trace_id: self.trace_id,
            other_uids: self.other_uids,
            origin: self.origin,
            name: self.name,
            result: self.result,
            request_params: self.request_params,
            result_params: self.result_params,
        }
    }
}
impl ser::Serialize for AuditLogV2 {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut size = 4usize;
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
        let skip_other_uids = self.other_uids.is_empty();
        if !skip_other_uids {
            size += 1;
        }
        let skip_origin = self.origin.is_none();
        if !skip_origin {
            size += 1;
        }
        let skip_request_params = self.request_params.is_empty();
        if !skip_request_params {
            size += 1;
        }
        let skip_result_params = self.result_params.is_empty();
        if !skip_result_params {
            size += 1;
        }
        let mut s = s.serialize_struct("AuditLogV2", size)?;
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
        if skip_trace_id {
            s.skip_field("traceId")?;
        } else {
            s.serialize_field("traceId", &self.trace_id)?;
        }
        if skip_other_uids {
            s.skip_field("otherUids")?;
        } else {
            s.serialize_field("otherUids", &self.other_uids)?;
        }
        if skip_origin {
            s.skip_field("origin")?;
        } else {
            s.serialize_field("origin", &self.origin)?;
        }
        s.serialize_field("name", &self.name)?;
        s.serialize_field("result", &self.result)?;
        if skip_request_params {
            s.skip_field("requestParams")?;
        } else {
            s.serialize_field("requestParams", &self.request_params)?;
        }
        if skip_result_params {
            s.skip_field("resultParams")?;
        } else {
            s.serialize_field("resultParams", &self.result_params)?;
        }
        s.end()
    }
}
impl<'de> de::Deserialize<'de> for AuditLogV2 {
    fn deserialize<D>(d: D) -> Result<AuditLogV2, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_struct(
            "AuditLogV2",
            &[
                "type",
                "time",
                "uid",
                "sid",
                "tokenId",
                "traceId",
                "otherUids",
                "origin",
                "name",
                "result",
                "requestParams",
                "resultParams",
            ],
            Visitor_,
        )
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = AuditLogV2;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("map")
    }
    fn visit_map<A>(self, mut map_: A) -> Result<AuditLogV2, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut type_ = None;
        let mut time = None;
        let mut uid = None;
        let mut sid = None;
        let mut token_id = None;
        let mut trace_id = None;
        let mut other_uids = None;
        let mut origin = None;
        let mut name = None;
        let mut result = None;
        let mut request_params = None;
        let mut result_params = None;
        while let Some(field_) = map_.next_key()? {
            match field_ {
                Field_::Type => type_ = Some(map_.next_value()?),
                Field_::Time => time = Some(map_.next_value()?),
                Field_::Uid => uid = Some(map_.next_value()?),
                Field_::Sid => sid = Some(map_.next_value()?),
                Field_::TokenId => token_id = Some(map_.next_value()?),
                Field_::TraceId => trace_id = Some(map_.next_value()?),
                Field_::OtherUids => other_uids = Some(map_.next_value()?),
                Field_::Origin => origin = Some(map_.next_value()?),
                Field_::Name => name = Some(map_.next_value()?),
                Field_::Result => result = Some(map_.next_value()?),
                Field_::RequestParams => request_params = Some(map_.next_value()?),
                Field_::ResultParams => result_params = Some(map_.next_value()?),
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
        let trace_id = match trace_id {
            Some(v) => v,
            None => Default::default(),
        };
        let other_uids = match other_uids {
            Some(v) => v,
            None => Default::default(),
        };
        let origin = match origin {
            Some(v) => v,
            None => Default::default(),
        };
        let name = match name {
            Some(v) => v,
            None => return Err(de::Error::missing_field("name")),
        };
        let result = match result {
            Some(v) => v,
            None => return Err(de::Error::missing_field("result")),
        };
        let request_params = match request_params {
            Some(v) => v,
            None => Default::default(),
        };
        let result_params = match result_params {
            Some(v) => v,
            None => Default::default(),
        };
        Ok(AuditLogV2 {
            type_,
            time,
            uid,
            sid,
            token_id,
            trace_id,
            other_uids,
            origin,
            name,
            result,
            request_params,
            result_params,
        })
    }
}
enum Field_ {
    Type,
    Time,
    Uid,
    Sid,
    TokenId,
    TraceId,
    OtherUids,
    Origin,
    Name,
    Result,
    RequestParams,
    ResultParams,
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
            "traceId" => Field_::TraceId,
            "otherUids" => Field_::OtherUids,
            "origin" => Field_::Origin,
            "name" => Field_::Name,
            "result" => Field_::Result,
            "requestParams" => Field_::RequestParams,
            "resultParams" => Field_::ResultParams,
            _ => Field_::Unknown_,
        };
        Ok(v)
    }
}
