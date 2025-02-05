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
pub struct SensitivityTaggedValue {
    #[builder(default, list(item(type = String, into)))]
    #[serde(rename = "level", skip_serializing_if = "Vec::is_empty", default)]
    level: Vec<String>,
    #[builder(
        custom(
            type = impl
            conjure_object::serde::Serialize,
            convert = |v|conjure_object::Any::new(v).expect("value failed to serialize")
        )
    )]
    #[serde(rename = "payload")]
    payload: conjure_object::Any,
}
impl SensitivityTaggedValue {
    /// Constructs a new instance of the type.
    #[inline]
    pub fn new(payload: impl conjure_object::serde::Serialize) -> Self {
        Self::builder().payload(payload).build()
    }
    ///Sensitivity level of this value; must be a known level in sls-spec.
    #[inline]
    pub fn level(&self) -> &[String] {
        &*self.level
    }
    #[inline]
    pub fn payload(&self) -> &conjure_object::Any {
        &self.payload
    }
}
