/// Wraps a log entry with metadata on where it is coming from and the source service that generated it.
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
pub struct WitchcraftEnvelopeV1 {
    #[builder(into)]
    #[serde(rename = "type")]
    type_: String,
    #[builder(into)]
    #[serde(rename = "deployment")]
    deployment: String,
    #[builder(into)]
    #[serde(rename = "environment")]
    environment: String,
    #[builder(into)]
    #[serde(rename = "environmentId")]
    environment_id: String,
    #[builder(into)]
    #[serde(rename = "host")]
    host: String,
    #[builder(into)]
    #[serde(rename = "nodeId")]
    node_id: String,
    #[builder(into)]
    #[serde(rename = "service")]
    service: String,
    #[builder(into)]
    #[serde(rename = "serviceId")]
    service_id: String,
    #[builder(into)]
    #[serde(rename = "stack")]
    stack: String,
    #[builder(into)]
    #[serde(rename = "stackId")]
    stack_id: String,
    #[builder(into)]
    #[serde(rename = "product")]
    product: String,
    #[builder(into)]
    #[serde(rename = "productVersion")]
    product_version: String,
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
impl WitchcraftEnvelopeV1 {
    /// "envelope.1"
    #[inline]
    pub fn type_(&self) -> &str {
        &*self.type_
    }
    /// Color or other codename for the customer infra
    #[inline]
    pub fn deployment(&self) -> &str {
        &*self.deployment
    }
    /// prod/staging/integration etc.
    #[inline]
    pub fn environment(&self) -> &str {
        &*self.environment
    }
    /// Skylab environment ID
    #[inline]
    pub fn environment_id(&self) -> &str {
        &*self.environment_id
    }
    /// Hostname where the log message originated
    #[inline]
    pub fn host(&self) -> &str {
        &*self.host
    }
    /// Skylab node ID
    #[inline]
    pub fn node_id(&self) -> &str {
        &*self.node_id
    }
    /// Skylab service name
    #[inline]
    pub fn service(&self) -> &str {
        &*self.service
    }
    /// Skylab service ID
    #[inline]
    pub fn service_id(&self) -> &str {
        &*self.service_id
    }
    /// Skylab stack name
    #[inline]
    pub fn stack(&self) -> &str {
        &*self.stack
    }
    /// Skylab stack ID
    #[inline]
    pub fn stack_id(&self) -> &str {
        &*self.stack_id
    }
    /// Artifact part of product's maven coordinate
    #[inline]
    pub fn product(&self) -> &str {
        &*self.product
    }
    /// Artifact semantic version
    #[inline]
    pub fn product_version(&self) -> &str {
        &*self.product_version
    }
    /// One of the Witchcraft log types; see [witchcraft-api](https://github.com/palantir/witchcraft-api) for details.
    #[inline]
    pub fn payload(&self) -> &conjure_object::Any {
        &self.payload
    }
}
