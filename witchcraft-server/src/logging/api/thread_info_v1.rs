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
pub struct ThreadInfoV1 {
    #[builder(default, into)]
    #[serde(rename = "id", skip_serializing_if = "Option::is_none", default)]
    id: Option<conjure_object::SafeLong>,
    #[builder(default, into)]
    #[serde(rename = "name", skip_serializing_if = "Option::is_none", default)]
    name: Option<String>,
    #[builder(default, list(item(type = super::StackFrameV1)))]
    #[serde(rename = "stackTrace", skip_serializing_if = "Vec::is_empty", default)]
    stack_trace: Vec<super::StackFrameV1>,
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
impl ThreadInfoV1 {
    /// Constructs a new instance of the type.
    #[inline]
    pub fn new() -> Self {
        Self::builder().build()
    }
    ///The ID of the thread.
    #[inline]
    pub fn id(&self) -> Option<conjure_object::SafeLong> {
        self.id.as_ref().map(|o| *o)
    }
    ///The name of the thread. Note that thread names may include unsafe information such as the path of the HTTP request being processed. It must be safely redacted.
    #[inline]
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|o| &**o)
    }
    ///A list of stack frames for the thread, ordered with the current frame first.
    #[inline]
    pub fn stack_trace(&self) -> &[super::StackFrameV1] {
        &*self.stack_trace
    }
    ///Other thread-level information.
    #[inline]
    pub fn params(&self) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.params
    }
}
