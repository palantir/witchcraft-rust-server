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
pub struct AuditLogV3 {
    #[builder(into)]
    #[serde(rename = "type")]
    type_: String,
    #[builder(into)]
    #[serde(rename = "deployment")]
    deployment: String,
    #[builder(into)]
    #[serde(rename = "host")]
    host: String,
    #[builder(into)]
    #[serde(rename = "product")]
    product: String,
    #[builder(into)]
    #[serde(rename = "productVersion")]
    product_version: String,
    #[builder(default, into)]
    #[serde(rename = "stack", skip_serializing_if = "Option::is_none", default)]
    stack: Option<String>,
    #[builder(default, into)]
    #[serde(rename = "service", skip_serializing_if = "Option::is_none", default)]
    service: Option<String>,
    #[builder(default, into)]
    #[serde(rename = "environment", skip_serializing_if = "Option::is_none", default)]
    environment: Option<String>,
    #[serde(rename = "producerType")]
    producer_type: super::AuditProducer,
    #[builder(default, list(item(type = super::Organization)))]
    #[serde(rename = "organizations", skip_serializing_if = "Vec::is_empty", default)]
    organizations: Vec<super::Organization>,
    #[serde(rename = "eventId")]
    event_id: conjure_object::Uuid,
    #[builder(default, into)]
    #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none", default)]
    user_agent: Option<String>,
    #[builder(default, list(item(type = String, into)))]
    #[serde(rename = "categories", skip_serializing_if = "Vec::is_empty", default)]
    categories: Vec<String>,
    #[builder(
        default,
        list(
            item(
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
    #[serde(rename = "entities", skip_serializing_if = "Vec::is_empty", default)]
    entities: Vec<conjure_object::Any>,
    #[builder(default, list(item(type = super::ContextualizedUser)))]
    #[serde(rename = "users", skip_serializing_if = "Vec::is_empty", default)]
    users: Vec<super::ContextualizedUser>,
    #[builder(default, list(item(type = String, into)))]
    #[serde(rename = "origins", skip_serializing_if = "Vec::is_empty", default)]
    origins: Vec<String>,
    #[builder(default, into)]
    #[serde(rename = "sourceOrigin", skip_serializing_if = "Option::is_none", default)]
    source_origin: Option<String>,
    #[builder(
        default,
        map(key(type = String, into), value(type = super::SensitivityTaggedValue))
    )]
    #[serde(
        rename = "requestParams",
        skip_serializing_if = "std::collections::BTreeMap::is_empty",
        default
    )]
    request_params: std::collections::BTreeMap<String, super::SensitivityTaggedValue>,
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
        rename = "requestFields",
        skip_serializing_if = "std::collections::BTreeMap::is_empty",
        default
    )]
    request_fields: std::collections::BTreeMap<String, conjure_object::Any>,
    #[builder(
        default,
        map(key(type = String, into), value(type = super::SensitivityTaggedValue))
    )]
    #[serde(
        rename = "resultParams",
        skip_serializing_if = "std::collections::BTreeMap::is_empty",
        default
    )]
    result_params: std::collections::BTreeMap<String, super::SensitivityTaggedValue>,
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
        rename = "resultFields",
        skip_serializing_if = "std::collections::BTreeMap::is_empty",
        default
    )]
    result_fields: std::collections::BTreeMap<String, conjure_object::Any>,
    #[serde(rename = "time")]
    time: conjure_object::DateTime<conjure_object::Utc>,
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
    #[builder(default, into)]
    #[serde(rename = "origin", skip_serializing_if = "Option::is_none", default)]
    origin: Option<String>,
    #[builder(into)]
    #[serde(rename = "name")]
    name: String,
    #[serde(rename = "result")]
    result: super::AuditResult,
}
impl AuditLogV3 {
    ///"audit.3"
    #[inline]
    pub fn type_(&self) -> &str {
        &*self.type_
    }
    ///The deployment that produced this log. Not exposed to downstream consumers.
    #[inline]
    pub fn deployment(&self) -> &str {
        &*self.deployment
    }
    ///The host of the service that produced this log.
    #[inline]
    pub fn host(&self) -> &str {
        &*self.host
    }
    ///The name of the product that produced this log.
    #[inline]
    pub fn product(&self) -> &str {
        &*self.product
    }
    ///The version of the product that produced this log.
    #[inline]
    pub fn product_version(&self) -> &str {
        &*self.product_version
    }
    ///The stack that this log was generated on.
    #[inline]
    pub fn stack(&self) -> Option<&str> {
        self.stack.as_ref().map(|o| &**o)
    }
    ///The service name that produced this log.
    #[inline]
    pub fn service(&self) -> Option<&str> {
        self.service.as_ref().map(|o| &**o)
    }
    ///The environment that produced this log.
    #[inline]
    pub fn environment(&self) -> Option<&str> {
        self.environment.as_ref().map(|o| &**o)
    }
    ///How this audit log was produced, eg. from a backend Server, frontend Client etc.
    #[inline]
    pub fn producer_type(&self) -> &super::AuditProducer {
        &self.producer_type
    }
    ///A list of organizations that have been attributed to this log.
    ///Attribution is typically based on the user that originated this log, and the resources that
    ///they targeted.
    ///Not exposed to downstream consumers.
    #[inline]
    pub fn organizations(&self) -> &[super::Organization] {
        &*self.organizations
    }
    ///Unique identifier for this audit log event.
    #[inline]
    pub fn event_id(&self) -> conjure_object::Uuid {
        self.event_id
    }
    ///The user agent of the user that originated this log.
    #[inline]
    pub fn user_agent(&self) -> Option<&str> {
        self.user_agent.as_ref().map(|o| &**o)
    }
    ///All audit categories produced by this audit event.
    ///Each audit categories produces a set of keys that will be distributed between the request and
    ///response params.
    #[inline]
    pub fn categories(&self) -> &[String] {
        &*self.categories
    }
    ///All contextualized entities present in the request and response params of this log.
    ///Note: Some resources cannot be contextualized, and will not be included in this list as a result.
    #[inline]
    pub fn entities(&self) -> &[conjure_object::Any] {
        &*self.entities
    }
    ///All contextualized users present in the request and response params of this log, including the top level
    ///UUID of this log.
    #[inline]
    pub fn users(&self) -> &[super::ContextualizedUser] {
        &*self.users
    }
    ///All addresses attached to the request. Contains information
    ///from unreliable sources such as the X-Forwarded-For header.
    ///
    ///This value can be spoofed.
    #[inline]
    pub fn origins(&self) -> &[String] {
        &*self.origins
    }
    ///Origin of the network request. If a request goes through a proxy,
    ///this will contain the proxy''s address.
    ///
    ///This value is verified through the TCP stack.
    #[inline]
    pub fn source_origin(&self) -> Option<&str> {
        self.source_origin.as_ref().map(|o| &**o)
    }
    #[deprecated(
        note = "Use requestFields instead.\n\nShould be translated to requestFields during emitting if requestFields is missing, by dropping the level\nfrom the SensitivityTaggedValue and directly using the payload as the value for the map.\n"
    )]
    #[inline]
    pub fn request_params(
        &self,
    ) -> &std::collections::BTreeMap<String, super::SensitivityTaggedValue> {
        &self.request_params
    }
    ///The fields known at method invocation time.
    ///
    ///Note that all keys must be known to the audit library. Typically, entries in the request and result
    ///fields will be dependent on the `categories` field defined above.
    ///
    ///This replaces requestParams and will take priority if present.
    #[inline]
    pub fn request_fields(
        &self,
    ) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.request_fields
    }
    #[deprecated(
        note = "Use resultFields instead.\n\nShould be translated to resultFields during emitting if resultFields is missing, by dropping the level\nfrom the SensitivityTaggedValue and directly using the payload as the value for the map.\n"
    )]
    #[inline]
    pub fn result_params(
        &self,
    ) -> &std::collections::BTreeMap<String, super::SensitivityTaggedValue> {
        &self.result_params
    }
    ///Information derived within a method, commonly parts of the return value.
    ///
    ///Note that all keys must be known to the audit library. Typically, entries in the request and result
    ///fields will be dependent on the `categories` field defined above.
    ///
    ///This replaces resultParams and will take priority if present.
    #[inline]
    pub fn result_fields(
        &self,
    ) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.result_fields
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
    ///Best-effort identifier of the originating machine, e.g. an
    ///IP address, a Kubernetes node identifier, or similar.
    ///
    ///This value can be spoofed.
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
}
