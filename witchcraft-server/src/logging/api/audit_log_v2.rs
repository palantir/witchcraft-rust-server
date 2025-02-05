///Definition of the audit.2 format.
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
pub struct AuditLogV2 {
    #[builder(into)]
    #[serde(rename = "type")]
    type_: String,
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
    #[builder(default, list(item(type = super::UserId)))]
    #[serde(rename = "otherUids", skip_serializing_if = "Vec::is_empty", default)]
    other_uids: Vec<super::UserId>,
    #[builder(default, into)]
    #[serde(rename = "origin", skip_serializing_if = "Option::is_none", default)]
    origin: Option<String>,
    #[builder(into)]
    #[serde(rename = "name")]
    name: String,
    #[serde(rename = "result")]
    result: super::AuditResult,
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
        rename = "requestParams",
        skip_serializing_if = "std::collections::BTreeMap::is_empty",
        default
    )]
    request_params: std::collections::BTreeMap<String, conjure_object::Any>,
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
        rename = "resultParams",
        skip_serializing_if = "std::collections::BTreeMap::is_empty",
        default
    )]
    result_params: std::collections::BTreeMap<String, conjure_object::Any>,
}
impl AuditLogV2 {
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
    pub fn request_params(
        &self,
    ) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.request_params
    }
    ///Information derived within a method, commonly parts of the return value.
    #[inline]
    pub fn result_params(
        &self,
    ) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.result_params
    }
}
