/// Definition of the service.1 format.
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
pub struct ServiceLogV1 {
    #[builder(into)]
    #[serde(rename = "type")]
    type_: String,
    #[serde(rename = "level")]
    level: super::LogLevel,
    #[serde(rename = "time")]
    time: conjure_object::DateTime<conjure_object::Utc>,
    #[builder(default, into)]
    #[serde(rename = "origin", skip_serializing_if = "Option::is_none", default)]
    origin: Option<String>,
    #[builder(default, into)]
    #[serde(rename = "thread", skip_serializing_if = "Option::is_none", default)]
    thread: Option<String>,
    #[builder(into)]
    #[serde(rename = "message")]
    message: String,
    #[builder(default, into)]
    #[serde(rename = "safe", skip_serializing_if = "Option::is_none", default)]
    safe: Option<bool>,
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
    #[serde(rename = "stacktrace", skip_serializing_if = "Option::is_none", default)]
    stacktrace: Option<String>,
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
    #[builder(default, map(key(type = String, into), value(type = String, into)))]
    #[serde(
        rename = "tags",
        skip_serializing_if = "std::collections::BTreeMap::is_empty",
        default
    )]
    tags: std::collections::BTreeMap<String, String>,
}
impl ServiceLogV1 {
    /// "service.1"
    #[inline]
    pub fn type_(&self) -> &str {
        &*self.type_
    }
    /// The logger output level. One of {FATAL,ERROR,WARN,INFO,DEBUG,TRACE} based on [log level coding guidelines](https://github.com/palantir/gradle-baseline/blob/develop/docs/best-practices/java-coding-guidelines/readme.md#log-levels)
    #[inline]
    pub fn level(&self) -> &super::LogLevel {
        &self.level
    }
    /// RFC3339Nano UTC datetime string when the log event was emitted
    #[inline]
    pub fn time(&self) -> conjure_object::DateTime<conjure_object::Utc> {
        self.time
    }
    /// Class or file name. May include line number.
    #[inline]
    pub fn origin(&self) -> Option<&str> {
        self.origin.as_ref().map(|o| &**o)
    }
    /// Thread name
    #[inline]
    pub fn thread(&self) -> Option<&str> {
        self.thread.as_ref().map(|o| &**o)
    }
    /// Log message. Palantir Java services using slf4j should not use slf4j placeholders ({}). Logs obtained from 3rd party libraries or services that use slf4j and contain slf4j placeholders will always produce `unsafeParams` with numeric indexes corresponding to the zero-indexed order of placeholders. Renderers should substitute numeric parameters from `unsafeParams` and may leave placeholders that do not match indexes as the original placeholder text.
    #[inline]
    pub fn message(&self) -> &str {
        &*self.message
    }
    /// Describes the safety of this log event based on prior knowledge within the application which produced the message. This field should not be set to `true` without _total_ confidence that it is correct. * _empty_:  Considered unsafe unless the logging pipeline has special configuration for this `origin`. Eventually these will all be equivalent to `false`. * `true`: All safe components can be trusted. * `false`: Event is _unsafe_ and cannot be exported.
    #[inline]
    pub fn safe(&self) -> Option<bool> {
        self.safe.as_ref().map(|o| *o)
    }
    /// Known-safe parameters (redaction may be used to make params knowably safe, but is not required).
    #[inline]
    pub fn params(&self) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.params
    }
    /// User id (if available).
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
    /// Language-specific stack trace. Content is knowably safe. Renderers should substitute named placeholders ({name}, for name as a key) with keyed value from unsafeParams and leave non-matching keys as the original placeholder text.
    #[inline]
    pub fn stacktrace(&self) -> Option<&str> {
        self.stacktrace.as_ref().map(|o| &**o)
    }
    /// Unredacted parameters
    #[inline]
    pub fn unsafe_params(
        &self,
    ) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.unsafe_params
    }
    /// Additional dimensions that describe the instance of the log event
    #[inline]
    pub fn tags(&self) -> &std::collections::BTreeMap<String, String> {
        &self.tags
    }
}
