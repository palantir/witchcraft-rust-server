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
pub struct ThreadDumpV1 {
    #[builder(default, list(item(type = super::ThreadInfoV1)))]
    #[serde(rename = "threads", skip_serializing_if = "Vec::is_empty", default)]
    threads: Vec<super::ThreadInfoV1>,
}
impl ThreadDumpV1 {
    /// Constructs a new instance of the type.
    #[inline]
    pub fn new() -> Self {
        Self::builder().build()
    }
    ///Information about each of the threads in the thread dump. "Thread" may refer to a userland thread such as a goroutine, or an OS-level thread.
    #[inline]
    pub fn threads(&self) -> &[super::ThreadInfoV1] {
        &*self.threads
    }
}
