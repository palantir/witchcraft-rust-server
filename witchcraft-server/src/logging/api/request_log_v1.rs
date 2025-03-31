/// Definition of the request.1 format.
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
pub struct RequestLogV1 {
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
        rename = "pathParams",
        skip_serializing_if = "std::collections::BTreeMap::is_empty",
        default
    )]
    path_params: std::collections::BTreeMap<String, conjure_object::Any>,
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
        rename = "queryParams",
        skip_serializing_if = "std::collections::BTreeMap::is_empty",
        default
    )]
    query_params: std::collections::BTreeMap<String, conjure_object::Any>,
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
        rename = "headerParams",
        skip_serializing_if = "std::collections::BTreeMap::is_empty",
        default
    )]
    header_params: std::collections::BTreeMap<String, conjure_object::Any>,
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
        rename = "bodyParams",
        skip_serializing_if = "std::collections::BTreeMap::is_empty",
        default
    )]
    body_params: std::collections::BTreeMap<String, conjure_object::Any>,
    #[serde(rename = "status")]
    status: i32,
    #[builder(into)]
    #[serde(rename = "requestSize")]
    request_size: String,
    #[builder(into)]
    #[serde(rename = "responseSize")]
    response_size: String,
    #[serde(rename = "duration")]
    duration: i32,
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
impl RequestLogV1 {
    #[inline]
    pub fn type_(&self) -> &str {
        &*self.type_
    }
    #[inline]
    pub fn time(&self) -> conjure_object::DateTime<conjure_object::Utc> {
        self.time
    }
    /// HTTP method of request
    #[inline]
    pub fn method(&self) -> Option<&str> {
        self.method.as_ref().map(|o| &**o)
    }
    /// Protocol, e.g. `HTTP/1.1`, `HTTP/2`
    #[inline]
    pub fn protocol(&self) -> &str {
        &*self.protocol
    }
    /// Path of request. If templated, the unrendered path, e.g.: `/catalog/dataset/{datasetId}`, `/{rid}/paths/contents/{path:.*}`.
    #[inline]
    pub fn path(&self) -> &str {
        &*self.path
    }
    /// Known-safe path parameters
    #[inline]
    pub fn path_params(
        &self,
    ) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.path_params
    }
    /// Known-safe query parameters
    #[inline]
    pub fn query_params(
        &self,
    ) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.query_params
    }
    /// Known-safe header parameters
    #[inline]
    pub fn header_params(
        &self,
    ) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.header_params
    }
    /// Known-safe body parameters
    #[inline]
    pub fn body_params(
        &self,
    ) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.body_params
    }
    /// HTTP status code of response
    #[inline]
    pub fn status(&self) -> i32 {
        self.status
    }
    /// Size of request (bytes). string to allow large numbers.
    #[inline]
    pub fn request_size(&self) -> &str {
        &*self.request_size
    }
    /// Size of response (bytes). string to allow large numbers.
    #[inline]
    pub fn response_size(&self) -> &str {
        &*self.response_size
    }
    /// Amount of time spent handling request (microseconds)
    #[inline]
    pub fn duration(&self) -> i32 {
        self.duration
    }
    /// User id (if available)
    #[inline]
    pub fn uid(&self) -> Option<&super::UserId> {
        self.uid.as_ref().map(|o| &*o)
    }
    /// Session id (if available)
    #[inline]
    pub fn sid(&self) -> Option<&super::SessionId> {
        self.sid.as_ref().map(|o| &*o)
    }
    /// API token id (if available)
    #[inline]
    pub fn token_id(&self) -> Option<&super::TokenId> {
        self.token_id.as_ref().map(|o| &*o)
    }
    /// Organization id (if available)
    #[inline]
    pub fn org_id(&self) -> Option<&super::OrganizationId> {
        self.org_id.as_ref().map(|o| &*o)
    }
    /// Zipkin trace id (if available)
    #[inline]
    pub fn trace_id(&self) -> Option<&super::TraceId> {
        self.trace_id.as_ref().map(|o| &*o)
    }
    /// Unredacted parameters such as path, query and header parameters
    #[inline]
    pub fn unsafe_params(
        &self,
    ) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.unsafe_params
    }
}
