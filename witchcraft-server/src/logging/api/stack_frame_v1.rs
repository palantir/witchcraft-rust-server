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
pub struct StackFrameV1 {
    #[builder(default, into)]
    #[serde(rename = "address", skip_serializing_if = "Option::is_none", default)]
    address: Option<String>,
    #[builder(default, into)]
    #[serde(rename = "procedure", skip_serializing_if = "Option::is_none", default)]
    procedure: Option<String>,
    #[builder(default, into)]
    #[serde(rename = "file", skip_serializing_if = "Option::is_none", default)]
    file: Option<String>,
    #[builder(default, into)]
    #[serde(rename = "line", skip_serializing_if = "Option::is_none", default)]
    line: Option<i32>,
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
impl StackFrameV1 {
    /// Constructs a new instance of the type.
    #[inline]
    pub fn new() -> Self {
        Self::builder().build()
    }
    /// The address of the execution point of this stack frame. This is a string because a safelong can't represent the full 64 bit address space.
    #[inline]
    pub fn address(&self) -> Option<&str> {
        self.address.as_ref().map(|o| &**o)
    }
    /// The identifier of the procedure containing the execution point of this stack frame. This is a fully qualified method name in Java and a demangled symbol name in native code, for example. Note that procedure names may include unsafe information if a service is, for exmaple, running user-defined code. It must be safely redacted.
    #[inline]
    pub fn procedure(&self) -> Option<&str> {
        self.procedure.as_ref().map(|o| &**o)
    }
    /// The name of the file containing the source location of the execution point of this stack frame. Note that file names may include unsafe information if a service is, for example, running user-defined code. It must be safely redacted.
    #[inline]
    pub fn file(&self) -> Option<&str> {
        self.file.as_ref().map(|o| &**o)
    }
    /// The line number of the source location of the execution point of this stack frame.
    #[inline]
    pub fn line(&self) -> Option<i32> {
        self.line.as_ref().map(|o| *o)
    }
    /// Other frame-level information.
    #[inline]
    pub fn params(&self) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.params
    }
}
