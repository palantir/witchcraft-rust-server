use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use conjure_object::serde::{de, ser};
use std::fmt;
#[doc = "Wraps a log entry with metadata on where it is coming from and the source service that generated it."]
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct WitchcraftEnvelopeV1 {
    type_: String,
    deployment: String,
    environment: String,
    environment_id: String,
    host: String,
    node_id: String,
    service: String,
    service_id: String,
    stack: String,
    stack_id: String,
    product: String,
    product_version: String,
    payload: conjure_object::Any,
}
impl WitchcraftEnvelopeV1 {
    #[doc = r" Returns a new builder."]
    #[inline]
    pub fn builder() -> BuilderStage0 {
        Default::default()
    }
    #[doc = "\"envelope.1\""]
    #[inline]
    pub fn type_(&self) -> &str {
        &*self.type_
    }
    #[doc = "Color or other codename for the customer infra"]
    #[inline]
    pub fn deployment(&self) -> &str {
        &*self.deployment
    }
    #[doc = "prod/staging/integration etc."]
    #[inline]
    pub fn environment(&self) -> &str {
        &*self.environment
    }
    #[doc = "Skylab environment ID"]
    #[inline]
    pub fn environment_id(&self) -> &str {
        &*self.environment_id
    }
    #[doc = "Hostname where the log message originated"]
    #[inline]
    pub fn host(&self) -> &str {
        &*self.host
    }
    #[doc = "Skylab node ID"]
    #[inline]
    pub fn node_id(&self) -> &str {
        &*self.node_id
    }
    #[doc = "Skylab service name"]
    #[inline]
    pub fn service(&self) -> &str {
        &*self.service
    }
    #[doc = "Skylab service ID"]
    #[inline]
    pub fn service_id(&self) -> &str {
        &*self.service_id
    }
    #[doc = "Skylab stack name"]
    #[inline]
    pub fn stack(&self) -> &str {
        &*self.stack
    }
    #[doc = "Skylab stack ID"]
    #[inline]
    pub fn stack_id(&self) -> &str {
        &*self.stack_id
    }
    #[doc = "Artifact part of product's maven coordinate"]
    #[inline]
    pub fn product(&self) -> &str {
        &*self.product
    }
    #[doc = "Artifact semantic version"]
    #[inline]
    pub fn product_version(&self) -> &str {
        &*self.product_version
    }
    #[doc = "One of the Witchcraft log types; see [witchcraft-api](https://github.com/palantir/witchcraft-api) for details."]
    #[inline]
    pub fn payload(&self) -> &conjure_object::Any {
        &self.payload
    }
}
impl Default for BuilderStage0 {
    #[inline]
    fn default() -> Self {
        BuilderStage0 {}
    }
}
impl From<WitchcraftEnvelopeV1> for BuilderStage13 {
    #[inline]
    fn from(value: WitchcraftEnvelopeV1) -> Self {
        BuilderStage13 {
            type_: value.type_,
            deployment: value.deployment,
            environment: value.environment,
            environment_id: value.environment_id,
            host: value.host,
            node_id: value.node_id,
            service: value.service,
            service_id: value.service_id,
            stack: value.stack,
            stack_id: value.stack_id,
            product: value.product,
            product_version: value.product_version,
            payload: value.payload,
        }
    }
}
#[doc = "The stage 0 builder for the [`WitchcraftEnvelopeV1`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage0 {}
impl BuilderStage0 {
    #[doc = "\"envelope.1\""]
    #[inline]
    pub fn type_<T>(self, type_: T) -> BuilderStage1
    where
        T: Into<String>,
    {
        BuilderStage1 {
            type_: type_.into(),
        }
    }
}
#[doc = "The stage 1 builder for the [`WitchcraftEnvelopeV1`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage1 {
    type_: String,
}
impl BuilderStage1 {
    #[doc = "Color or other codename for the customer infra"]
    #[inline]
    pub fn deployment<T>(self, deployment: T) -> BuilderStage2
    where
        T: Into<String>,
    {
        BuilderStage2 {
            type_: self.type_,
            deployment: deployment.into(),
        }
    }
}
#[doc = "The stage 2 builder for the [`WitchcraftEnvelopeV1`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage2 {
    type_: String,
    deployment: String,
}
impl BuilderStage2 {
    #[doc = "prod/staging/integration etc."]
    #[inline]
    pub fn environment<T>(self, environment: T) -> BuilderStage3
    where
        T: Into<String>,
    {
        BuilderStage3 {
            type_: self.type_,
            deployment: self.deployment,
            environment: environment.into(),
        }
    }
}
#[doc = "The stage 3 builder for the [`WitchcraftEnvelopeV1`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage3 {
    type_: String,
    deployment: String,
    environment: String,
}
impl BuilderStage3 {
    #[doc = "Skylab environment ID"]
    #[inline]
    pub fn environment_id<T>(self, environment_id: T) -> BuilderStage4
    where
        T: Into<String>,
    {
        BuilderStage4 {
            type_: self.type_,
            deployment: self.deployment,
            environment: self.environment,
            environment_id: environment_id.into(),
        }
    }
}
#[doc = "The stage 4 builder for the [`WitchcraftEnvelopeV1`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage4 {
    type_: String,
    deployment: String,
    environment: String,
    environment_id: String,
}
impl BuilderStage4 {
    #[doc = "Hostname where the log message originated"]
    #[inline]
    pub fn host<T>(self, host: T) -> BuilderStage5
    where
        T: Into<String>,
    {
        BuilderStage5 {
            type_: self.type_,
            deployment: self.deployment,
            environment: self.environment,
            environment_id: self.environment_id,
            host: host.into(),
        }
    }
}
#[doc = "The stage 5 builder for the [`WitchcraftEnvelopeV1`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage5 {
    type_: String,
    deployment: String,
    environment: String,
    environment_id: String,
    host: String,
}
impl BuilderStage5 {
    #[doc = "Skylab node ID"]
    #[inline]
    pub fn node_id<T>(self, node_id: T) -> BuilderStage6
    where
        T: Into<String>,
    {
        BuilderStage6 {
            type_: self.type_,
            deployment: self.deployment,
            environment: self.environment,
            environment_id: self.environment_id,
            host: self.host,
            node_id: node_id.into(),
        }
    }
}
#[doc = "The stage 6 builder for the [`WitchcraftEnvelopeV1`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage6 {
    type_: String,
    deployment: String,
    environment: String,
    environment_id: String,
    host: String,
    node_id: String,
}
impl BuilderStage6 {
    #[doc = "Skylab service name"]
    #[inline]
    pub fn service<T>(self, service: T) -> BuilderStage7
    where
        T: Into<String>,
    {
        BuilderStage7 {
            type_: self.type_,
            deployment: self.deployment,
            environment: self.environment,
            environment_id: self.environment_id,
            host: self.host,
            node_id: self.node_id,
            service: service.into(),
        }
    }
}
#[doc = "The stage 7 builder for the [`WitchcraftEnvelopeV1`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage7 {
    type_: String,
    deployment: String,
    environment: String,
    environment_id: String,
    host: String,
    node_id: String,
    service: String,
}
impl BuilderStage7 {
    #[doc = "Skylab service ID"]
    #[inline]
    pub fn service_id<T>(self, service_id: T) -> BuilderStage8
    where
        T: Into<String>,
    {
        BuilderStage8 {
            type_: self.type_,
            deployment: self.deployment,
            environment: self.environment,
            environment_id: self.environment_id,
            host: self.host,
            node_id: self.node_id,
            service: self.service,
            service_id: service_id.into(),
        }
    }
}
#[doc = "The stage 8 builder for the [`WitchcraftEnvelopeV1`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage8 {
    type_: String,
    deployment: String,
    environment: String,
    environment_id: String,
    host: String,
    node_id: String,
    service: String,
    service_id: String,
}
impl BuilderStage8 {
    #[doc = "Skylab stack name"]
    #[inline]
    pub fn stack<T>(self, stack: T) -> BuilderStage9
    where
        T: Into<String>,
    {
        BuilderStage9 {
            type_: self.type_,
            deployment: self.deployment,
            environment: self.environment,
            environment_id: self.environment_id,
            host: self.host,
            node_id: self.node_id,
            service: self.service,
            service_id: self.service_id,
            stack: stack.into(),
        }
    }
}
#[doc = "The stage 9 builder for the [`WitchcraftEnvelopeV1`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage9 {
    type_: String,
    deployment: String,
    environment: String,
    environment_id: String,
    host: String,
    node_id: String,
    service: String,
    service_id: String,
    stack: String,
}
impl BuilderStage9 {
    #[doc = "Skylab stack ID"]
    #[inline]
    pub fn stack_id<T>(self, stack_id: T) -> BuilderStage10
    where
        T: Into<String>,
    {
        BuilderStage10 {
            type_: self.type_,
            deployment: self.deployment,
            environment: self.environment,
            environment_id: self.environment_id,
            host: self.host,
            node_id: self.node_id,
            service: self.service,
            service_id: self.service_id,
            stack: self.stack,
            stack_id: stack_id.into(),
        }
    }
}
#[doc = "The stage 10 builder for the [`WitchcraftEnvelopeV1`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage10 {
    type_: String,
    deployment: String,
    environment: String,
    environment_id: String,
    host: String,
    node_id: String,
    service: String,
    service_id: String,
    stack: String,
    stack_id: String,
}
impl BuilderStage10 {
    #[doc = "Artifact part of product's maven coordinate"]
    #[inline]
    pub fn product<T>(self, product: T) -> BuilderStage11
    where
        T: Into<String>,
    {
        BuilderStage11 {
            type_: self.type_,
            deployment: self.deployment,
            environment: self.environment,
            environment_id: self.environment_id,
            host: self.host,
            node_id: self.node_id,
            service: self.service,
            service_id: self.service_id,
            stack: self.stack,
            stack_id: self.stack_id,
            product: product.into(),
        }
    }
}
#[doc = "The stage 11 builder for the [`WitchcraftEnvelopeV1`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage11 {
    type_: String,
    deployment: String,
    environment: String,
    environment_id: String,
    host: String,
    node_id: String,
    service: String,
    service_id: String,
    stack: String,
    stack_id: String,
    product: String,
}
impl BuilderStage11 {
    #[doc = "Artifact semantic version"]
    #[inline]
    pub fn product_version<T>(self, product_version: T) -> BuilderStage12
    where
        T: Into<String>,
    {
        BuilderStage12 {
            type_: self.type_,
            deployment: self.deployment,
            environment: self.environment,
            environment_id: self.environment_id,
            host: self.host,
            node_id: self.node_id,
            service: self.service,
            service_id: self.service_id,
            stack: self.stack,
            stack_id: self.stack_id,
            product: self.product,
            product_version: product_version.into(),
        }
    }
}
#[doc = "The stage 12 builder for the [`WitchcraftEnvelopeV1`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage12 {
    type_: String,
    deployment: String,
    environment: String,
    environment_id: String,
    host: String,
    node_id: String,
    service: String,
    service_id: String,
    stack: String,
    stack_id: String,
    product: String,
    product_version: String,
}
impl BuilderStage12 {
    #[doc = "One of the Witchcraft log types; see [witchcraft-api](https://github.com/palantir/witchcraft-api) for details."]
    #[inline]
    pub fn payload<T>(self, payload: T) -> BuilderStage13
    where
        T: conjure_object::serde::Serialize,
    {
        BuilderStage13 {
            type_: self.type_,
            deployment: self.deployment,
            environment: self.environment,
            environment_id: self.environment_id,
            host: self.host,
            node_id: self.node_id,
            service: self.service,
            service_id: self.service_id,
            stack: self.stack,
            stack_id: self.stack_id,
            product: self.product,
            product_version: self.product_version,
            payload: conjure_object::Any::new(payload).expect("value failed to serialize"),
        }
    }
}
#[doc = "The stage 13 builder for the [`WitchcraftEnvelopeV1`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage13 {
    type_: String,
    deployment: String,
    environment: String,
    environment_id: String,
    host: String,
    node_id: String,
    service: String,
    service_id: String,
    stack: String,
    stack_id: String,
    product: String,
    product_version: String,
    payload: conjure_object::Any,
}
impl BuilderStage13 {
    #[doc = "\"envelope.1\""]
    #[inline]
    pub fn type_<T>(mut self, type_: T) -> Self
    where
        T: Into<String>,
    {
        self.type_ = type_.into();
        self
    }
    #[doc = "Color or other codename for the customer infra"]
    #[inline]
    pub fn deployment<T>(mut self, deployment: T) -> Self
    where
        T: Into<String>,
    {
        self.deployment = deployment.into();
        self
    }
    #[doc = "prod/staging/integration etc."]
    #[inline]
    pub fn environment<T>(mut self, environment: T) -> Self
    where
        T: Into<String>,
    {
        self.environment = environment.into();
        self
    }
    #[doc = "Skylab environment ID"]
    #[inline]
    pub fn environment_id<T>(mut self, environment_id: T) -> Self
    where
        T: Into<String>,
    {
        self.environment_id = environment_id.into();
        self
    }
    #[doc = "Hostname where the log message originated"]
    #[inline]
    pub fn host<T>(mut self, host: T) -> Self
    where
        T: Into<String>,
    {
        self.host = host.into();
        self
    }
    #[doc = "Skylab node ID"]
    #[inline]
    pub fn node_id<T>(mut self, node_id: T) -> Self
    where
        T: Into<String>,
    {
        self.node_id = node_id.into();
        self
    }
    #[doc = "Skylab service name"]
    #[inline]
    pub fn service<T>(mut self, service: T) -> Self
    where
        T: Into<String>,
    {
        self.service = service.into();
        self
    }
    #[doc = "Skylab service ID"]
    #[inline]
    pub fn service_id<T>(mut self, service_id: T) -> Self
    where
        T: Into<String>,
    {
        self.service_id = service_id.into();
        self
    }
    #[doc = "Skylab stack name"]
    #[inline]
    pub fn stack<T>(mut self, stack: T) -> Self
    where
        T: Into<String>,
    {
        self.stack = stack.into();
        self
    }
    #[doc = "Skylab stack ID"]
    #[inline]
    pub fn stack_id<T>(mut self, stack_id: T) -> Self
    where
        T: Into<String>,
    {
        self.stack_id = stack_id.into();
        self
    }
    #[doc = "Artifact part of product's maven coordinate"]
    #[inline]
    pub fn product<T>(mut self, product: T) -> Self
    where
        T: Into<String>,
    {
        self.product = product.into();
        self
    }
    #[doc = "Artifact semantic version"]
    #[inline]
    pub fn product_version<T>(mut self, product_version: T) -> Self
    where
        T: Into<String>,
    {
        self.product_version = product_version.into();
        self
    }
    #[doc = "One of the Witchcraft log types; see [witchcraft-api](https://github.com/palantir/witchcraft-api) for details."]
    #[inline]
    pub fn payload<T>(mut self, payload: T) -> Self
    where
        T: conjure_object::serde::Serialize,
    {
        self.payload = conjure_object::Any::new(payload).expect("value failed to serialize");
        self
    }
    #[doc = r" Consumes the builder, constructing a new instance of the type."]
    #[inline]
    pub fn build(self) -> WitchcraftEnvelopeV1 {
        WitchcraftEnvelopeV1 {
            type_: self.type_,
            deployment: self.deployment,
            environment: self.environment,
            environment_id: self.environment_id,
            host: self.host,
            node_id: self.node_id,
            service: self.service,
            service_id: self.service_id,
            stack: self.stack,
            stack_id: self.stack_id,
            product: self.product,
            product_version: self.product_version,
            payload: self.payload,
        }
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
