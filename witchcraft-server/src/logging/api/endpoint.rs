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
pub struct Endpoint {
    #[builder(into)]
    #[serde(rename = "serviceName")]
    service_name: String,
    #[builder(default, into)]
    #[serde(rename = "ipv4", skip_serializing_if = "Option::is_none", default)]
    ipv4: Option<String>,
    #[builder(default, into)]
    #[serde(rename = "ipv6", skip_serializing_if = "Option::is_none", default)]
    ipv6: Option<String>,
}
impl Endpoint {
    /// Constructs a new instance of the type.
    #[inline]
    pub fn new(service_name: impl Into<String>) -> Self {
        Self::builder().service_name(service_name).build()
    }
    /// Name of the service that generated the annotation
    #[inline]
    pub fn service_name(&self) -> &str {
        &*self.service_name
    }
    /// IPv4 address of the machine that generated this annotation (`xxx.xxx.xxx.xxx`)
    #[inline]
    pub fn ipv4(&self) -> Option<&str> {
        self.ipv4.as_ref().map(|o| &**o)
    }
    /// IPv6 address of the machine that generated this annotation (standard hextet form)
    #[inline]
    pub fn ipv6(&self) -> Option<&str> {
        self.ipv6.as_ref().map(|o| &**o)
    }
}
