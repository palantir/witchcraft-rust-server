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
pub struct GenericDiagnostic {
    #[builder(into)]
    #[serde(rename = "diagnosticType")]
    diagnostic_type: String,
    #[builder(
        custom(
            type = impl
            conjure_object::serde::Serialize,
            convert = |v|conjure_object::Any::new(v).expect("value failed to serialize")
        )
    )]
    #[serde(rename = "value")]
    value: conjure_object::Any,
}
impl GenericDiagnostic {
    /// Constructs a new instance of the type.
    #[inline]
    pub fn new(
        diagnostic_type: impl Into<String>,
        value: impl conjure_object::serde::Serialize,
    ) -> Self {
        Self::builder().diagnostic_type(diagnostic_type).value(value).build()
    }
    /// An identifier for the type of diagnostic represented.
    #[inline]
    pub fn diagnostic_type(&self) -> &str {
        &*self.diagnostic_type
    }
    /// Observations, measurements and context associated with the diagnostic.
    #[inline]
    pub fn value(&self) -> &conjure_object::Any {
        &self.value
    }
}
