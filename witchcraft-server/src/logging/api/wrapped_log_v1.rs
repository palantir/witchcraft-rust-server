/// Wraps a log entry with entity information.
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
pub struct WrappedLogV1 {
    #[builder(into)]
    #[serde(rename = "type")]
    type_: String,
    #[builder(custom(type = super::WrappedLogV1Payload, convert = Box::new))]
    #[serde(rename = "payload")]
    payload: Box<super::WrappedLogV1Payload>,
    #[builder(into)]
    #[serde(rename = "entityName")]
    entity_name: String,
    #[builder(into)]
    #[serde(rename = "entityVersion")]
    entity_version: String,
    #[builder(default, into)]
    #[serde(rename = "service", skip_serializing_if = "Option::is_none", default)]
    service: Option<String>,
    #[builder(default, into)]
    #[serde(rename = "serviceId", skip_serializing_if = "Option::is_none", default)]
    service_id: Option<String>,
    #[builder(default, into)]
    #[serde(rename = "stack", skip_serializing_if = "Option::is_none", default)]
    stack: Option<String>,
    #[builder(default, into)]
    #[serde(rename = "stackId", skip_serializing_if = "Option::is_none", default)]
    stack_id: Option<String>,
}
impl WrappedLogV1 {
    /// "wrapped.1"
    #[inline]
    pub fn type_(&self) -> &str {
        &*self.type_
    }
    #[inline]
    pub fn payload(&self) -> &super::WrappedLogV1Payload {
        &*self.payload
    }
    /// Artifact part of entity's maven coordinate
    #[inline]
    pub fn entity_name(&self) -> &str {
        &*self.entity_name
    }
    #[inline]
    pub fn entity_version(&self) -> &str {
        &*self.entity_version
    }
    /// Defaults to the wrapped log producer's Skylab service name.
    #[inline]
    pub fn service(&self) -> Option<&str> {
        self.service.as_ref().map(|o| &**o)
    }
    /// Defaults to the wrapped log producer's Skylab service ID.
    #[inline]
    pub fn service_id(&self) -> Option<&str> {
        self.service_id.as_ref().map(|o| &**o)
    }
    /// Defaults to the wrapped log producer's Skylab stack name.
    #[inline]
    pub fn stack(&self) -> Option<&str> {
        self.stack.as_ref().map(|o| &**o)
    }
    /// Defaults to the wrapped log producer's Skylab stack ID.
    #[inline]
    pub fn stack_id(&self) -> Option<&str> {
        self.stack_id.as_ref().map(|o| &**o)
    }
}
