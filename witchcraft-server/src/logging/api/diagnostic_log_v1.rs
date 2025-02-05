///Definition of the diagnostic.1 format.
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
pub struct DiagnosticLogV1 {
    #[builder(into)]
    #[serde(rename = "type")]
    type_: String,
    #[serde(rename = "time")]
    time: conjure_object::DateTime<conjure_object::Utc>,
    #[builder(custom(type = super::Diagnostic, convert = Box::new))]
    #[serde(rename = "diagnostic")]
    diagnostic: Box<super::Diagnostic>,
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
impl DiagnosticLogV1 {
    /// Constructs a new instance of the type.
    #[inline]
    pub fn new(
        type_: impl Into<String>,
        time: conjure_object::DateTime<conjure_object::Utc>,
        diagnostic: super::Diagnostic,
    ) -> Self {
        Self::builder().type_(type_).time(time).diagnostic(diagnostic).build()
    }
    ///"diagnostic.1"
    #[inline]
    pub fn type_(&self) -> &str {
        &*self.type_
    }
    #[inline]
    pub fn time(&self) -> conjure_object::DateTime<conjure_object::Utc> {
        self.time
    }
    ///The diagnostic being logged.
    #[inline]
    pub fn diagnostic(&self) -> &super::Diagnostic {
        &*self.diagnostic
    }
    ///Unredacted parameters
    #[inline]
    pub fn unsafe_params(
        &self,
    ) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.unsafe_params
    }
}
