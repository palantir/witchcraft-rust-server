use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
///Definition of the request.2 format.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[conjure_object::private::staged_builder::staged_builder]
#[builder(crate = conjure_object::private::staged_builder, update, inline)]
pub struct RequestLogV2 {
    #[builder(into)]
    type_: String,
    time: conjure_object::DateTime<conjure_object::Utc>,
    #[builder(default, into)]
    method: Option<String>,
    #[builder(into)]
    protocol: String,
    #[builder(into)]
    path: String,
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
    status: i32,
    request_size: conjure_object::SafeLong,
    response_size: conjure_object::SafeLong,
    duration: conjure_object::SafeLong,
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
}
impl RequestLogV2 {
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
    ///Unredacted parameters such as path, query and header parameters
    #[inline]
    pub fn unsafe_params(
        &self,
    ) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.unsafe_params
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
        let skip_org_id = self.org_id.is_none();
        if !skip_org_id {
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
                "orgId",
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
        let mut org_id = None;
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
                Field_::OrgId => org_id = Some(map_.next_value()?),
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
        let org_id = match org_id {
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
            org_id,
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
    OrgId,
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
            "orgId" => Field_::OrgId,
            "traceId" => Field_::TraceId,
            "unsafeParams" => Field_::UnsafeParams,
            _ => Field_::Unknown_,
        };
        Ok(v)
    }
}
