///Definition of the request.2 format.
#[derive(
    Debug,
    Clone,
    conjure_object::serde::Serialize,
    conjure_object::serde::Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash
)]
#[serde(crate = "conjure_object::serde")]
#[conjure_object::private::staged_builder::staged_builder]
#[builder(crate = conjure_object::private::staged_builder, update, inline)]
pub struct RequestLogV2 {
    #[builder(into)]
    #[serde(rename = "type")]
    type_: String,
    #[serde(rename = "time")]
    time: conjure_object::DateTime<conjure_object::Utc>,
    #[builder(default, into)]
    #[serde(rename = "method", skip_serializing_if = "Option::is_none", default)]
    method: Option<String>,
    #[builder(into)]
    #[serde(rename = "protocol")]
    protocol: String,
    #[builder(into)]
    #[serde(rename = "path")]
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
    #[serde(
        rename = "params",
        skip_serializing_if = "std::collections::BTreeMap::is_empty",
        default
    )]
    params: std::collections::BTreeMap<String, conjure_object::Any>,
    #[serde(rename = "status")]
    status: i32,
    #[serde(rename = "requestSize")]
    request_size: conjure_object::SafeLong,
    #[serde(rename = "responseSize")]
    response_size: conjure_object::SafeLong,
    #[serde(rename = "duration")]
    duration: conjure_object::SafeLong,
    #[builder(default, into)]
    #[serde(rename = "uid", skip_serializing_if = "Option::is_none", default)]
    uid: Option<super::UserId>,
    #[builder(default, into)]
    #[serde(rename = "sid", skip_serializing_if = "Option::is_none", default)]
    sid: Option<super::SessionId>,
    #[builder(default, into)]
    #[serde(rename = "tokenId", skip_serializing_if = "Option::is_none", default)]
    token_id: Option<super::TokenId>,
    #[builder(default, into)]
    #[serde(rename = "orgId", skip_serializing_if = "Option::is_none", default)]
    org_id: Option<super::OrganizationId>,
    #[builder(default, into)]
    #[serde(rename = "traceId", skip_serializing_if = "Option::is_none", default)]
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
    #[serde(
        rename = "unsafeParams",
        skip_serializing_if = "std::collections::BTreeMap::is_empty",
        default
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
