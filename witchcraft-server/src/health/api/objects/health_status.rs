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
pub struct HealthStatus {
    #[builder(
        default,
        map(key(type = super::CheckType), value(type = super::HealthCheckResult))
    )]
    #[serde(
        rename = "checks",
        skip_serializing_if = "std::collections::BTreeMap::is_empty",
        default
    )]
    checks: std::collections::BTreeMap<super::CheckType, super::HealthCheckResult>,
}
impl HealthStatus {
    /// Constructs a new instance of the type.
    #[inline]
    pub fn new() -> Self {
        Self::builder().build()
    }
    #[inline]
    pub fn checks(
        &self,
    ) -> &std::collections::BTreeMap<super::CheckType, super::HealthCheckResult> {
        &self.checks
    }
}
