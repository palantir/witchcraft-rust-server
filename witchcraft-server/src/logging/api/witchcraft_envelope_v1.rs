use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
///Wraps a log entry with metadata on where it is coming from and the source service that generated it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[conjure_object::private::staged_builder::staged_builder]
#[builder(crate = conjure_object::private::staged_builder, update, inline)]
pub struct WitchcraftEnvelopeV1 {
    #[builder(into)]
    type_: String,
    #[builder(into)]
    deployment: String,
    #[builder(into)]
    environment: String,
    #[builder(into)]
    environment_id: String,
    #[builder(into)]
    host: String,
    #[builder(into)]
    node_id: String,
    #[builder(into)]
    service: String,
    #[builder(into)]
    service_id: String,
    #[builder(into)]
    stack: String,
    #[builder(into)]
    stack_id: String,
    #[builder(into)]
    product: String,
    #[builder(into)]
    product_version: String,
    #[builder(
        custom(
            type = impl
            conjure_object::serde::Serialize,
            convert = |v|conjure_object::Any::new(v).expect("value failed to serialize")
        )
    )]
    payload: conjure_object::Any,
}
impl WitchcraftEnvelopeV1 {
    ///"envelope.1"
    #[inline]
    pub fn type_(&self) -> &str {
        &*self.type_
    }
    ///Color or other codename for the customer infra
    #[inline]
    pub fn deployment(&self) -> &str {
        &*self.deployment
    }
    ///prod/staging/integration etc.
    #[inline]
    pub fn environment(&self) -> &str {
        &*self.environment
    }
    ///Skylab environment ID
    #[inline]
    pub fn environment_id(&self) -> &str {
        &*self.environment_id
    }
    ///Hostname where the log message originated
    #[inline]
    pub fn host(&self) -> &str {
        &*self.host
    }
    ///Skylab node ID
    #[inline]
    pub fn node_id(&self) -> &str {
        &*self.node_id
    }
    ///Skylab service name
    #[inline]
    pub fn service(&self) -> &str {
        &*self.service
    }
    ///Skylab service ID
    #[inline]
    pub fn service_id(&self) -> &str {
        &*self.service_id
    }
    ///Skylab stack name
    #[inline]
    pub fn stack(&self) -> &str {
        &*self.stack
    }
    ///Skylab stack ID
    #[inline]
    pub fn stack_id(&self) -> &str {
        &*self.stack_id
    }
    ///Artifact part of product's maven coordinate
    #[inline]
    pub fn product(&self) -> &str {
        &*self.product
    }
    ///Artifact semantic version
    #[inline]
    pub fn product_version(&self) -> &str {
        &*self.product_version
    }
    ///One of the Witchcraft log types; see [witchcraft-api](https://github.com/palantir/witchcraft-api) for details.
    #[inline]
    pub fn payload(&self) -> &conjure_object::Any {
        &self.payload
    }
}
impl ser::Serialize for WitchcraftEnvelopeV1 {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let size = 13usize;
        let mut s = s.serialize_struct("WitchcraftEnvelopeV1", size)?;
        s.serialize_field("type", &self.type_)?;
        s.serialize_field("deployment", &self.deployment)?;
        s.serialize_field("environment", &self.environment)?;
        s.serialize_field("environmentId", &self.environment_id)?;
        s.serialize_field("host", &self.host)?;
        s.serialize_field("nodeId", &self.node_id)?;
        s.serialize_field("service", &self.service)?;
        s.serialize_field("serviceId", &self.service_id)?;
        s.serialize_field("stack", &self.stack)?;
        s.serialize_field("stackId", &self.stack_id)?;
        s.serialize_field("product", &self.product)?;
        s.serialize_field("productVersion", &self.product_version)?;
        s.serialize_field("payload", &self.payload)?;
        s.end()
    }
}
impl<'de> de::Deserialize<'de> for WitchcraftEnvelopeV1 {
    fn deserialize<D>(d: D) -> Result<WitchcraftEnvelopeV1, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_struct(
            "WitchcraftEnvelopeV1",
            &[
                "type",
                "deployment",
                "environment",
                "environmentId",
                "host",
                "nodeId",
                "service",
                "serviceId",
                "stack",
                "stackId",
                "product",
                "productVersion",
                "payload",
            ],
            Visitor_,
        )
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = WitchcraftEnvelopeV1;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("map")
    }
    fn visit_map<A>(self, mut map_: A) -> Result<WitchcraftEnvelopeV1, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut type_ = None;
        let mut deployment = None;
        let mut environment = None;
        let mut environment_id = None;
        let mut host = None;
        let mut node_id = None;
        let mut service = None;
        let mut service_id = None;
        let mut stack = None;
        let mut stack_id = None;
        let mut product = None;
        let mut product_version = None;
        let mut payload = None;
        while let Some(field_) = map_.next_key()? {
            match field_ {
                Field_::Type => type_ = Some(map_.next_value()?),
                Field_::Deployment => deployment = Some(map_.next_value()?),
                Field_::Environment => environment = Some(map_.next_value()?),
                Field_::EnvironmentId => environment_id = Some(map_.next_value()?),
                Field_::Host => host = Some(map_.next_value()?),
                Field_::NodeId => node_id = Some(map_.next_value()?),
                Field_::Service => service = Some(map_.next_value()?),
                Field_::ServiceId => service_id = Some(map_.next_value()?),
                Field_::Stack => stack = Some(map_.next_value()?),
                Field_::StackId => stack_id = Some(map_.next_value()?),
                Field_::Product => product = Some(map_.next_value()?),
                Field_::ProductVersion => product_version = Some(map_.next_value()?),
                Field_::Payload => payload = Some(map_.next_value()?),
                Field_::Unknown_ => {
                    map_.next_value::<de::IgnoredAny>()?;
                }
            }
        }
        let type_ = match type_ {
            Some(v) => v,
            None => return Err(de::Error::missing_field("type")),
        };
        let deployment = match deployment {
            Some(v) => v,
            None => return Err(de::Error::missing_field("deployment")),
        };
        let environment = match environment {
            Some(v) => v,
            None => return Err(de::Error::missing_field("environment")),
        };
        let environment_id = match environment_id {
            Some(v) => v,
            None => return Err(de::Error::missing_field("environmentId")),
        };
        let host = match host {
            Some(v) => v,
            None => return Err(de::Error::missing_field("host")),
        };
        let node_id = match node_id {
            Some(v) => v,
            None => return Err(de::Error::missing_field("nodeId")),
        };
        let service = match service {
            Some(v) => v,
            None => return Err(de::Error::missing_field("service")),
        };
        let service_id = match service_id {
            Some(v) => v,
            None => return Err(de::Error::missing_field("serviceId")),
        };
        let stack = match stack {
            Some(v) => v,
            None => return Err(de::Error::missing_field("stack")),
        };
        let stack_id = match stack_id {
            Some(v) => v,
            None => return Err(de::Error::missing_field("stackId")),
        };
        let product = match product {
            Some(v) => v,
            None => return Err(de::Error::missing_field("product")),
        };
        let product_version = match product_version {
            Some(v) => v,
            None => return Err(de::Error::missing_field("productVersion")),
        };
        let payload = match payload {
            Some(v) => v,
            None => return Err(de::Error::missing_field("payload")),
        };
        Ok(WitchcraftEnvelopeV1 {
            type_,
            deployment,
            environment,
            environment_id,
            host,
            node_id,
            service,
            service_id,
            stack,
            stack_id,
            product,
            product_version,
            payload,
        })
    }
}
enum Field_ {
    Type,
    Deployment,
    Environment,
    EnvironmentId,
    Host,
    NodeId,
    Service,
    ServiceId,
    Stack,
    StackId,
    Product,
    ProductVersion,
    Payload,
    Unknown_,
}
impl<'de> de::Deserialize<'de> for Field_ {
    fn deserialize<D>(d: D) -> Result<Field_, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_str(FieldVisitor_)
    }
}
struct FieldVisitor_;
impl<'de> de::Visitor<'de> for FieldVisitor_ {
    type Value = Field_;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("string")
    }
    fn visit_str<E>(self, value: &str) -> Result<Field_, E>
    where
        E: de::Error,
    {
        let v = match value {
            "type" => Field_::Type,
            "deployment" => Field_::Deployment,
            "environment" => Field_::Environment,
            "environmentId" => Field_::EnvironmentId,
            "host" => Field_::Host,
            "nodeId" => Field_::NodeId,
            "service" => Field_::Service,
            "serviceId" => Field_::ServiceId,
            "stack" => Field_::Stack,
            "stackId" => Field_::StackId,
            "product" => Field_::Product,
            "productVersion" => Field_::ProductVersion,
            "payload" => Field_::Payload,
            _ => Field_::Unknown_,
        };
        Ok(v)
    }
}
