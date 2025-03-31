/// A Zipkin-compatible Span object.
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
pub struct Span {
    #[builder(into)]
    #[serde(rename = "traceId")]
    trace_id: String,
    #[builder(into)]
    #[serde(rename = "id")]
    id: String,
    #[builder(into)]
    #[serde(rename = "name")]
    name: String,
    #[builder(default, into)]
    #[serde(rename = "parentId", skip_serializing_if = "Option::is_none", default)]
    parent_id: Option<String>,
    #[serde(rename = "timestamp")]
    timestamp: conjure_object::SafeLong,
    #[serde(rename = "duration")]
    duration: conjure_object::SafeLong,
    #[builder(default, list(item(type = super::Annotation)))]
    #[serde(rename = "annotations", skip_serializing_if = "Vec::is_empty", default)]
    annotations: Vec<super::Annotation>,
    #[builder(default, map(key(type = String, into), value(type = String, into)))]
    #[serde(
        rename = "tags",
        skip_serializing_if = "std::collections::BTreeMap::is_empty",
        default
    )]
    tags: std::collections::BTreeMap<String, String>,
}
impl Span {
    /// 16-digit hex trace identifier
    #[inline]
    pub fn trace_id(&self) -> &str {
        &*self.trace_id
    }
    /// 16-digit hex span identifier
    #[inline]
    pub fn id(&self) -> &str {
        &*self.id
    }
    /// Name of the span (typically the operation/RPC/method name for corresponding to this span)
    #[inline]
    pub fn name(&self) -> &str {
        &*self.name
    }
    /// 16-digit hex identifer of the parent span
    #[inline]
    pub fn parent_id(&self) -> Option<&str> {
        self.parent_id.as_ref().map(|o| &**o)
    }
    /// Timestamp of the start of this span (epoch microsecond value)
    #[inline]
    pub fn timestamp(&self) -> conjure_object::SafeLong {
        self.timestamp
    }
    /// Duration of this span (microseconds)
    #[inline]
    pub fn duration(&self) -> conjure_object::SafeLong {
        self.duration
    }
    #[inline]
    pub fn annotations(&self) -> &[super::Annotation] {
        &*self.annotations
    }
    /// Additional dimensions that describe the instance of the trace span
    #[inline]
    pub fn tags(&self) -> &std::collections::BTreeMap<String, String> {
        &self.tags
    }
}
