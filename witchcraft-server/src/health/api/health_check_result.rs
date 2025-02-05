///Metadata describing the status of a service.
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
pub struct HealthCheckResult {
    #[serde(rename = "type")]
    type_: super::CheckType,
    #[serde(rename = "state")]
    state: super::HealthState,
    #[builder(default, into)]
    #[serde(rename = "message", skip_serializing_if = "Option::is_none", default)]
    message: Option<String>,
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
}
impl HealthCheckResult {
    /// Constructs a new instance of the type.
    #[inline]
    pub fn new(type_: super::CheckType, state: super::HealthState) -> Self {
        Self::builder().type_(type_).state(state).build()
    }
    ///A constant representing the type of health check. Values should be uppercase, underscore delimited, ascii letters with no spaces, ([A-Z_]).
    #[inline]
    pub fn type_(&self) -> &super::CheckType {
        &self.type_
    }
    ///Health state of the check.
    #[inline]
    pub fn state(&self) -> &super::HealthState {
        &self.state
    }
    ///Text describing the state of the check which should provide enough information for the check to be actionable when included in an alert.
    #[inline]
    pub fn message(&self) -> Option<&str> {
        self.message.as_ref().map(|o| &**o)
    }
    ///Additional redacted information on the nature of the health check.
    #[inline]
    pub fn params(&self) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.params
    }
}
