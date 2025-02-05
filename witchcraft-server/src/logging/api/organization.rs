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
pub struct Organization {
    #[builder(into)]
    #[serde(rename = "id")]
    id: String,
    #[builder(into)]
    #[serde(rename = "reason")]
    reason: String,
}
impl Organization {
    /// Constructs a new instance of the type.
    #[inline]
    pub fn new(id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::builder().id(id).reason(reason).build()
    }
    ///Organization RID. Not exposed to downstream consumers.
    #[inline]
    pub fn id(&self) -> &str {
        &*self.id
    }
    ///Explanation of why this organization was attributed to this log.
    #[inline]
    pub fn reason(&self) -> &str {
        &*self.reason
    }
}
