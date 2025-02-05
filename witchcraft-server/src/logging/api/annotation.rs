///A Zipkin-compatible Annotation object.
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
pub struct Annotation {
    #[serde(rename = "timestamp")]
    timestamp: conjure_object::SafeLong,
    #[builder(into)]
    #[serde(rename = "value")]
    value: String,
    #[builder(custom(type = super::Endpoint, convert = Box::new))]
    #[serde(rename = "endpoint")]
    endpoint: Box<super::Endpoint>,
}
impl Annotation {
    /// Constructs a new instance of the type.
    #[inline]
    pub fn new(
        timestamp: conjure_object::SafeLong,
        value: impl Into<String>,
        endpoint: super::Endpoint,
    ) -> Self {
        Self::builder().timestamp(timestamp).value(value).endpoint(endpoint).build()
    }
    ///Time annotation was created (epoch microsecond value)
    #[inline]
    pub fn timestamp(&self) -> conjure_object::SafeLong {
        self.timestamp
    }
    ///Value encapsulated by this annotation
    #[inline]
    pub fn value(&self) -> &str {
        &*self.value
    }
    #[inline]
    pub fn endpoint(&self) -> &super::Endpoint {
        &*self.endpoint
    }
}
