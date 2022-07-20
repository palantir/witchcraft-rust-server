use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
///Definition of the request.2 format.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RequestLogV2 {
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
    method: Option<String>,
    protocol: String,
    path: String,
    params: std::collections::BTreeMap<String, conjure_object::Any>,
    status: i32,
    request_size: conjure_object::SafeLong,
    response_size: conjure_object::SafeLong,
    duration: conjure_object::SafeLong,
    uid: Option<super::UserId>,
    sid: Option<super::SessionId>,
    token_id: Option<super::TokenId>,
    trace_id: Option<super::TraceId>,
    unsafe_params: std::collections::BTreeMap<String, conjure_object::Any>,
}
impl RequestLogV2 {
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
    ///HTTP method of request
    #[inline]
    pub fn method(&self) -> Option<&str> {
        self.method.as_ref().map(|o| &**o)
    }
    ///Protocol, e.g. `HTTP/1.1`, `HTTP/2`
    #[inline]
    pub fn protocol(&self) -> &str {
        &*self.protocol
    }
    ///Path of request. If templated, the unrendered path, e.g.: `/catalog/dataset/{datasetId}`, `/{rid}/paths/contents/{path:.*}`.
    #[inline]
    pub fn path(&self) -> &str {
        &*self.path
    }
    ///Known-safe parameters
    #[inline]
    pub fn params(&self) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.params
    }
    ///HTTP status code of response
    #[inline]
    pub fn status(&self) -> i32 {
        self.status
    }
    ///Size of request (bytes)
    #[inline]
    pub fn request_size(&self) -> conjure_object::SafeLong {
        self.request_size
    }
    ///Size of response (bytes)
    #[inline]
    pub fn response_size(&self) -> conjure_object::SafeLong {
        self.response_size
    }
    ///Amount of time spent handling request (microseconds)
    #[inline]
    pub fn duration(&self) -> conjure_object::SafeLong {
        self.duration
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
    ///Unredacted parameters such as path, query and header parameters
    #[inline]
    pub fn unsafe_params(
        &self,
    ) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.unsafe_params
    }
}
impl Default for BuilderStage0 {
    #[inline]
    fn default() -> Self {
        BuilderStage0 {}
    }
}
impl From<RequestLogV2> for BuilderStage8 {
    #[inline]
    fn from(value: RequestLogV2) -> Self {
        BuilderStage8 {
            type_: value.type_,
            time: value.time,
            method: value.method,
            protocol: value.protocol,
            path: value.path,
            params: value.params,
            status: value.status,
            request_size: value.request_size,
            response_size: value.response_size,
            duration: value.duration,
            uid: value.uid,
            sid: value.sid,
            token_id: value.token_id,
            trace_id: value.trace_id,
            unsafe_params: value.unsafe_params,
        }
    }
}
///The stage 0 builder for the [`RequestLogV2`] type
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
///The stage 1 builder for the [`RequestLogV2`] type
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
///The stage 2 builder for the [`RequestLogV2`] type
#[derive(Debug, Clone)]
pub struct BuilderStage2 {
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
}
impl BuilderStage2 {
    ///Protocol, e.g. `HTTP/1.1`, `HTTP/2`
    #[inline]
    pub fn protocol<T>(self, protocol: T) -> BuilderStage3
    where
        T: Into<String>,
    {
        BuilderStage3 {
            type_: self.type_,
            time: self.time,
            protocol: protocol.into(),
        }
    }
}
///The stage 3 builder for the [`RequestLogV2`] type
#[derive(Debug, Clone)]
pub struct BuilderStage3 {
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
    protocol: String,
}
impl BuilderStage3 {
    ///Path of request. If templated, the unrendered path, e.g.: `/catalog/dataset/{datasetId}`, `/{rid}/paths/contents/{path:.*}`.
    #[inline]
    pub fn path<T>(self, path: T) -> BuilderStage4
    where
        T: Into<String>,
    {
        BuilderStage4 {
            type_: self.type_,
            time: self.time,
            protocol: self.protocol,
            path: path.into(),
        }
    }
}
///The stage 4 builder for the [`RequestLogV2`] type
#[derive(Debug, Clone)]
pub struct BuilderStage4 {
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
    protocol: String,
    path: String,
}
impl BuilderStage4 {
    ///HTTP status code of response
    #[inline]
    pub fn status(self, status: i32) -> BuilderStage5 {
        BuilderStage5 {
            type_: self.type_,
            time: self.time,
            protocol: self.protocol,
            path: self.path,
            status: status,
        }
    }
}
///The stage 5 builder for the [`RequestLogV2`] type
#[derive(Debug, Clone)]
pub struct BuilderStage5 {
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
    protocol: String,
    path: String,
    status: i32,
}
impl BuilderStage5 {
    ///Size of request (bytes)
    #[inline]
    pub fn request_size(self, request_size: conjure_object::SafeLong) -> BuilderStage6 {
        BuilderStage6 {
            type_: self.type_,
            time: self.time,
            protocol: self.protocol,
            path: self.path,
            status: self.status,
            request_size: request_size,
        }
    }
}
///The stage 6 builder for the [`RequestLogV2`] type
#[derive(Debug, Clone)]
pub struct BuilderStage6 {
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
    protocol: String,
    path: String,
    status: i32,
    request_size: conjure_object::SafeLong,
}
impl BuilderStage6 {
    ///Size of response (bytes)
    #[inline]
    pub fn response_size(
        self,
        response_size: conjure_object::SafeLong,
    ) -> BuilderStage7 {
        BuilderStage7 {
            type_: self.type_,
            time: self.time,
            protocol: self.protocol,
            path: self.path,
            status: self.status,
            request_size: self.request_size,
            response_size: response_size,
        }
    }
}
///The stage 7 builder for the [`RequestLogV2`] type
#[derive(Debug, Clone)]
pub struct BuilderStage7 {
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
    protocol: String,
    path: String,
    status: i32,
    request_size: conjure_object::SafeLong,
    response_size: conjure_object::SafeLong,
}
impl BuilderStage7 {
    ///Amount of time spent handling request (microseconds)
    #[inline]
    pub fn duration(self, duration: conjure_object::SafeLong) -> BuilderStage8 {
        BuilderStage8 {
            type_: self.type_,
            time: self.time,
            protocol: self.protocol,
            path: self.path,
            status: self.status,
            request_size: self.request_size,
            response_size: self.response_size,
            duration: duration,
            method: Default::default(),
            params: Default::default(),
            uid: Default::default(),
            sid: Default::default(),
            token_id: Default::default(),
            trace_id: Default::default(),
            unsafe_params: Default::default(),
        }
    }
}
///The stage 8 builder for the [`RequestLogV2`] type
#[derive(Debug, Clone)]
pub struct BuilderStage8 {
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
    protocol: String,
    path: String,
    status: i32,
    request_size: conjure_object::SafeLong,
    response_size: conjure_object::SafeLong,
    duration: conjure_object::SafeLong,
    method: Option<String>,
    params: std::collections::BTreeMap<String, conjure_object::Any>,
    uid: Option<super::UserId>,
    sid: Option<super::SessionId>,
    token_id: Option<super::TokenId>,
    trace_id: Option<super::TraceId>,
    unsafe_params: std::collections::BTreeMap<String, conjure_object::Any>,
}
impl BuilderStage8 {
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
    ///Protocol, e.g. `HTTP/1.1`, `HTTP/2`
    #[inline]
    pub fn protocol<T>(mut self, protocol: T) -> Self
    where
        T: Into<String>,
    {
        self.protocol = protocol.into();
        self
    }
    ///Path of request. If templated, the unrendered path, e.g.: `/catalog/dataset/{datasetId}`, `/{rid}/paths/contents/{path:.*}`.
    #[inline]
    pub fn path<T>(mut self, path: T) -> Self
    where
        T: Into<String>,
    {
        self.path = path.into();
        self
    }
    ///HTTP status code of response
    #[inline]
    pub fn status(mut self, status: i32) -> Self {
        self.status = status;
        self
    }
    ///Size of request (bytes)
    #[inline]
    pub fn request_size(mut self, request_size: conjure_object::SafeLong) -> Self {
        self.request_size = request_size;
        self
    }
    ///Size of response (bytes)
    #[inline]
    pub fn response_size(mut self, response_size: conjure_object::SafeLong) -> Self {
        self.response_size = response_size;
        self
    }
    ///Amount of time spent handling request (microseconds)
    #[inline]
    pub fn duration(mut self, duration: conjure_object::SafeLong) -> Self {
        self.duration = duration;
        self
    }
    ///HTTP method of request
    #[inline]
    pub fn method<T>(mut self, method: T) -> Self
    where
        T: Into<Option<String>>,
    {
        self.method = method.into();
        self
    }
    ///Known-safe parameters
    #[inline]
    pub fn params<T>(mut self, params: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.params = params.into_iter().collect();
        self
    }
    ///Known-safe parameters
    #[inline]
    pub fn extend_params<T>(mut self, params: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.params.extend(params);
        self
    }
    ///Known-safe parameters
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
    ///Unredacted parameters such as path, query and header parameters
    #[inline]
    pub fn unsafe_params<T>(mut self, unsafe_params: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.unsafe_params = unsafe_params.into_iter().collect();
        self
    }
    ///Unredacted parameters such as path, query and header parameters
    #[inline]
    pub fn extend_unsafe_params<T>(mut self, unsafe_params: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.unsafe_params.extend(unsafe_params);
        self
    }
    ///Unredacted parameters such as path, query and header parameters
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
    pub fn build(self) -> RequestLogV2 {
        RequestLogV2 {
            type_: self.type_,
            time: self.time,
            method: self.method,
            protocol: self.protocol,
            path: self.path,
            params: self.params,
            status: self.status,
            request_size: self.request_size,
            response_size: self.response_size,
            duration: self.duration,
            uid: self.uid,
            sid: self.sid,
            token_id: self.token_id,
            trace_id: self.trace_id,
            unsafe_params: self.unsafe_params,
        }
    }
}
impl ser::Serialize for RequestLogV2 {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut size = 8usize;
        let skip_method = self.method.is_none();
        if !skip_method {
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
        let skip_unsafe_params = self.unsafe_params.is_empty();
        if !skip_unsafe_params {
            size += 1;
        }
        let mut s = s.serialize_struct("RequestLogV2", size)?;
        s.serialize_field("type", &self.type_)?;
        s.serialize_field("time", &self.time)?;
        if skip_method {
            s.skip_field("method")?;
        } else {
            s.serialize_field("method", &self.method)?;
        }
        s.serialize_field("protocol", &self.protocol)?;
        s.serialize_field("path", &self.path)?;
        if skip_params {
            s.skip_field("params")?;
        } else {
            s.serialize_field("params", &self.params)?;
        }
        s.serialize_field("status", &self.status)?;
        s.serialize_field("requestSize", &self.request_size)?;
        s.serialize_field("responseSize", &self.response_size)?;
        s.serialize_field("duration", &self.duration)?;
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
        s.end()
    }
}
impl<'de> de::Deserialize<'de> for RequestLogV2 {
    fn deserialize<D>(d: D) -> Result<RequestLogV2, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_struct(
            "RequestLogV2",
            &[
                "type",
                "time",
                "method",
                "protocol",
                "path",
                "params",
                "status",
                "requestSize",
                "responseSize",
                "duration",
                "uid",
                "sid",
                "tokenId",
                "traceId",
                "unsafeParams",
            ],
            Visitor_,
        )
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = RequestLogV2;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("map")
    }
    fn visit_map<A>(self, mut map_: A) -> Result<RequestLogV2, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut type_ = None;
        let mut time = None;
        let mut method = None;
        let mut protocol = None;
        let mut path = None;
        let mut params = None;
        let mut status = None;
        let mut request_size = None;
        let mut response_size = None;
        let mut duration = None;
        let mut uid = None;
        let mut sid = None;
        let mut token_id = None;
        let mut trace_id = None;
        let mut unsafe_params = None;
        while let Some(field_) = map_.next_key()? {
            match field_ {
                Field_::Type => type_ = Some(map_.next_value()?),
                Field_::Time => time = Some(map_.next_value()?),
                Field_::Method => method = Some(map_.next_value()?),
                Field_::Protocol => protocol = Some(map_.next_value()?),
                Field_::Path => path = Some(map_.next_value()?),
                Field_::Params => params = Some(map_.next_value()?),
                Field_::Status => status = Some(map_.next_value()?),
                Field_::RequestSize => request_size = Some(map_.next_value()?),
                Field_::ResponseSize => response_size = Some(map_.next_value()?),
                Field_::Duration => duration = Some(map_.next_value()?),
                Field_::Uid => uid = Some(map_.next_value()?),
                Field_::Sid => sid = Some(map_.next_value()?),
                Field_::TokenId => token_id = Some(map_.next_value()?),
                Field_::TraceId => trace_id = Some(map_.next_value()?),
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
        let method = match method {
            Some(v) => v,
            None => Default::default(),
        };
        let protocol = match protocol {
            Some(v) => v,
            None => return Err(de::Error::missing_field("protocol")),
        };
        let path = match path {
            Some(v) => v,
            None => return Err(de::Error::missing_field("path")),
        };
        let params = match params {
            Some(v) => v,
            None => Default::default(),
        };
        let status = match status {
            Some(v) => v,
            None => return Err(de::Error::missing_field("status")),
        };
        let request_size = match request_size {
            Some(v) => v,
            None => return Err(de::Error::missing_field("requestSize")),
        };
        let response_size = match response_size {
            Some(v) => v,
            None => return Err(de::Error::missing_field("responseSize")),
        };
        let duration = match duration {
            Some(v) => v,
            None => return Err(de::Error::missing_field("duration")),
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
        Ok(RequestLogV2 {
            type_,
            time,
            method,
            protocol,
            path,
            params,
            status,
            request_size,
            response_size,
            duration,
            uid,
            sid,
            token_id,
            trace_id,
            unsafe_params,
        })
    }
}
enum Field_ {
    Type,
    Time,
    Method,
    Protocol,
    Path,
    Params,
    Status,
    RequestSize,
    ResponseSize,
    Duration,
    Uid,
    Sid,
    TokenId,
    TraceId,
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
            "method" => Field_::Method,
            "protocol" => Field_::Protocol,
            "path" => Field_::Path,
            "params" => Field_::Params,
            "status" => Field_::Status,
            "requestSize" => Field_::RequestSize,
            "responseSize" => Field_::ResponseSize,
            "duration" => Field_::Duration,
            "uid" => Field_::Uid,
            "sid" => Field_::Sid,
            "tokenId" => Field_::TokenId,
            "traceId" => Field_::TraceId,
            "unsafeParams" => Field_::UnsafeParams,
            _ => Field_::Unknown_,
        };
        Ok(v)
    }
}
