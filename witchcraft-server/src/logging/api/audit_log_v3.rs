use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AuditLogV3 {
    type_: String,
    deployment: String,
    host: String,
    product: String,
    product_version: String,
    stack: Option<String>,
    service: Option<String>,
    environment: Option<String>,
    producer_type: super::AuditProducer,
    organizations: Vec<super::Organization>,
    event_id: conjure_object::Uuid,
    user_agent: Option<String>,
    categories: Vec<String>,
    entities: Vec<conjure_object::Any>,
    users: Vec<super::ContextualizedUser>,
    origins: Vec<String>,
    source_origin: Option<String>,
    request_params: std::collections::BTreeMap<String, super::SensitivityTaggedValue>,
    result_params: std::collections::BTreeMap<String, super::SensitivityTaggedValue>,
    time: conjure_object::DateTime<conjure_object::Utc>,
    uid: Option<super::UserId>,
    sid: Option<super::SessionId>,
    token_id: Option<super::TokenId>,
    org_id: Option<super::OrganizationId>,
    trace_id: Option<super::TraceId>,
    origin: Option<String>,
    name: String,
    result: super::AuditResult,
}
impl AuditLogV3 {
    /// Returns a new builder.
    #[inline]
    pub fn builder() -> BuilderStage0 {
        Default::default()
    }
    ///"audit.3"
    #[inline]
    pub fn type_(&self) -> &str {
        &*self.type_
    }
    ///The deployment that produced this log. Not exposed to downstream consumers.
    #[inline]
    pub fn deployment(&self) -> &str {
        &*self.deployment
    }
    ///The host of the service that produced this log.
    #[inline]
    pub fn host(&self) -> &str {
        &*self.host
    }
    ///The name of the product that produced this log.
    #[inline]
    pub fn product(&self) -> &str {
        &*self.product
    }
    ///The version of the product that produced this log.
    #[inline]
    pub fn product_version(&self) -> &str {
        &*self.product_version
    }
    ///The stack that this log was generated on.
    #[inline]
    pub fn stack(&self) -> Option<&str> {
        self.stack.as_ref().map(|o| &**o)
    }
    ///The service name that produced this log.
    #[inline]
    pub fn service(&self) -> Option<&str> {
        self.service.as_ref().map(|o| &**o)
    }
    ///The environment that produced this log.
    #[inline]
    pub fn environment(&self) -> Option<&str> {
        self.environment.as_ref().map(|o| &**o)
    }
    ///How this audit log was produced, eg. from a backend Server, frontend Client etc.
    #[inline]
    pub fn producer_type(&self) -> &super::AuditProducer {
        &self.producer_type
    }
    ///A list of organizations that have been attributed to this log.
    ///Attribution is typically based on the user that originated this log, and the resources that
    ///they targeted.
    ///Not exposed to downstream consumers.
    #[inline]
    pub fn organizations(&self) -> &[super::Organization] {
        &*self.organizations
    }
    ///Unique identifier for this audit log event.
    #[inline]
    pub fn event_id(&self) -> conjure_object::Uuid {
        self.event_id
    }
    ///The user agent of the user that originated this log.
    #[inline]
    pub fn user_agent(&self) -> Option<&str> {
        self.user_agent.as_ref().map(|o| &**o)
    }
    ///All audit categories produced by this audit event.
    ///Each audit categories produces a set of keys that will be distributed between the request and
    ///response params.
    #[inline]
    pub fn categories(&self) -> &[String] {
        &*self.categories
    }
    ///All contextualized entities present in the request and response params of this log.
    ///Note: Some resources cannot be contextualized, and will not be included in this list as a result.
    #[inline]
    pub fn entities(&self) -> &[conjure_object::Any] {
        &*self.entities
    }
    ///All contextualized users present in the request and response params of this log, including the top level
    ///UUID of this log.
    #[inline]
    pub fn users(&self) -> &[super::ContextualizedUser] {
        &*self.users
    }
    ///All addresses attached to the request. Contains information
    ///from unreliable sources such as the X-Forwarded-For header.
    ///
    ///This value can be spoofed.
    #[inline]
    pub fn origins(&self) -> &[String] {
        &*self.origins
    }
    ///Origin of the network request. If a request goes through a proxy,
    ///this will contain the proxy''s address.
    ///
    ///This value is verified through the TCP stack.
    #[inline]
    pub fn source_origin(&self) -> Option<&str> {
        self.source_origin.as_ref().map(|o| &**o)
    }
    ///The parameters known at method invocation time.
    ///
    ///Note that all keys must be known to the audit library. Typically, entries in the request and response
    ///params will be dependent on the `categories` field defined above.
    #[inline]
    pub fn request_params(
        &self,
    ) -> &std::collections::BTreeMap<String, super::SensitivityTaggedValue> {
        &self.request_params
    }
    ///Information derived within a method, commonly parts of the return value.
    ///
    ///Note that all keys must be known to the audit library. Typically, entries in the request and response
    ///params will be dependent on the `categories` field defined above.
    #[inline]
    pub fn result_params(
        &self,
    ) -> &std::collections::BTreeMap<String, super::SensitivityTaggedValue> {
        &self.result_params
    }
    #[inline]
    pub fn time(&self) -> conjure_object::DateTime<conjure_object::Utc> {
        self.time
    }
    ///User id (if available). This is the most downstream caller.
    #[inline]
    pub fn uid(&self) -> Option<&super::UserId> {
        self.uid.as_ref().map(|o| &*o)
    }
    ///Session id (if available)
    #[inline]
    pub fn sid(&self) -> Option<&super::SessionId> {
        self.sid.as_ref().map(|o| &*o)
    }
    ///API token id (if available)
    #[inline]
    pub fn token_id(&self) -> Option<&super::TokenId> {
        self.token_id.as_ref().map(|o| &*o)
    }
    ///Organization id (if available)
    #[inline]
    pub fn org_id(&self) -> Option<&super::OrganizationId> {
        self.org_id.as_ref().map(|o| &*o)
    }
    ///Zipkin trace id (if available)
    #[inline]
    pub fn trace_id(&self) -> Option<&super::TraceId> {
        self.trace_id.as_ref().map(|o| &*o)
    }
    ///Best-effort identifier of the originating machine, e.g. an
    ///IP address, a Kubernetes node identifier, or similar.
    ///
    ///This value can be spoofed.
    #[inline]
    pub fn origin(&self) -> Option<&str> {
        self.origin.as_ref().map(|o| &**o)
    }
    ///Name of the audit event, e.g. PUT_FILE
    #[inline]
    pub fn name(&self) -> &str {
        &*self.name
    }
    ///Indicates whether the request was successful or the type of failure, e.g. ERROR or UNAUTHORIZED
    #[inline]
    pub fn result(&self) -> &super::AuditResult {
        &self.result
    }
}
impl Default for BuilderStage0 {
    #[inline]
    fn default() -> Self {
        BuilderStage0 {}
    }
}
impl From<AuditLogV3> for BuilderStage10 {
    #[inline]
    fn from(value: AuditLogV3) -> Self {
        BuilderStage10 {
            type_: value.type_,
            deployment: value.deployment,
            host: value.host,
            product: value.product,
            product_version: value.product_version,
            stack: value.stack,
            service: value.service,
            environment: value.environment,
            producer_type: value.producer_type,
            organizations: value.organizations,
            event_id: value.event_id,
            user_agent: value.user_agent,
            categories: value.categories,
            entities: value.entities,
            users: value.users,
            origins: value.origins,
            source_origin: value.source_origin,
            request_params: value.request_params,
            result_params: value.result_params,
            time: value.time,
            uid: value.uid,
            sid: value.sid,
            token_id: value.token_id,
            org_id: value.org_id,
            trace_id: value.trace_id,
            origin: value.origin,
            name: value.name,
            result: value.result,
        }
    }
}
///The stage 0 builder for the [`AuditLogV3`] type
#[derive(Debug, Clone)]
pub struct BuilderStage0 {}
impl BuilderStage0 {
    ///"audit.3"
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
///The stage 1 builder for the [`AuditLogV3`] type
#[derive(Debug, Clone)]
pub struct BuilderStage1 {
    type_: String,
}
impl BuilderStage1 {
    ///The deployment that produced this log. Not exposed to downstream consumers.
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
///The stage 2 builder for the [`AuditLogV3`] type
#[derive(Debug, Clone)]
pub struct BuilderStage2 {
    type_: String,
    deployment: String,
}
impl BuilderStage2 {
    ///The host of the service that produced this log.
    #[inline]
    pub fn host<T>(self, host: T) -> BuilderStage3
    where
        T: Into<String>,
    {
        BuilderStage3 {
            type_: self.type_,
            deployment: self.deployment,
            host: host.into(),
        }
    }
}
///The stage 3 builder for the [`AuditLogV3`] type
#[derive(Debug, Clone)]
pub struct BuilderStage3 {
    type_: String,
    deployment: String,
    host: String,
}
impl BuilderStage3 {
    ///The name of the product that produced this log.
    #[inline]
    pub fn product<T>(self, product: T) -> BuilderStage4
    where
        T: Into<String>,
    {
        BuilderStage4 {
            type_: self.type_,
            deployment: self.deployment,
            host: self.host,
            product: product.into(),
        }
    }
}
///The stage 4 builder for the [`AuditLogV3`] type
#[derive(Debug, Clone)]
pub struct BuilderStage4 {
    type_: String,
    deployment: String,
    host: String,
    product: String,
}
impl BuilderStage4 {
    ///The version of the product that produced this log.
    #[inline]
    pub fn product_version<T>(self, product_version: T) -> BuilderStage5
    where
        T: Into<String>,
    {
        BuilderStage5 {
            type_: self.type_,
            deployment: self.deployment,
            host: self.host,
            product: self.product,
            product_version: product_version.into(),
        }
    }
}
///The stage 5 builder for the [`AuditLogV3`] type
#[derive(Debug, Clone)]
pub struct BuilderStage5 {
    type_: String,
    deployment: String,
    host: String,
    product: String,
    product_version: String,
}
impl BuilderStage5 {
    ///How this audit log was produced, eg. from a backend Server, frontend Client etc.
    #[inline]
    pub fn producer_type(self, producer_type: super::AuditProducer) -> BuilderStage6 {
        BuilderStage6 {
            type_: self.type_,
            deployment: self.deployment,
            host: self.host,
            product: self.product,
            product_version: self.product_version,
            producer_type: producer_type,
        }
    }
}
///The stage 6 builder for the [`AuditLogV3`] type
#[derive(Debug, Clone)]
pub struct BuilderStage6 {
    type_: String,
    deployment: String,
    host: String,
    product: String,
    product_version: String,
    producer_type: super::AuditProducer,
}
impl BuilderStage6 {
    ///Unique identifier for this audit log event.
    #[inline]
    pub fn event_id(self, event_id: conjure_object::Uuid) -> BuilderStage7 {
        BuilderStage7 {
            type_: self.type_,
            deployment: self.deployment,
            host: self.host,
            product: self.product,
            product_version: self.product_version,
            producer_type: self.producer_type,
            event_id: event_id,
        }
    }
}
///The stage 7 builder for the [`AuditLogV3`] type
#[derive(Debug, Clone)]
pub struct BuilderStage7 {
    type_: String,
    deployment: String,
    host: String,
    product: String,
    product_version: String,
    producer_type: super::AuditProducer,
    event_id: conjure_object::Uuid,
}
impl BuilderStage7 {
    #[inline]
    pub fn time(
        self,
        time: conjure_object::DateTime<conjure_object::Utc>,
    ) -> BuilderStage8 {
        BuilderStage8 {
            type_: self.type_,
            deployment: self.deployment,
            host: self.host,
            product: self.product,
            product_version: self.product_version,
            producer_type: self.producer_type,
            event_id: self.event_id,
            time: time,
        }
    }
}
///The stage 8 builder for the [`AuditLogV3`] type
#[derive(Debug, Clone)]
pub struct BuilderStage8 {
    type_: String,
    deployment: String,
    host: String,
    product: String,
    product_version: String,
    producer_type: super::AuditProducer,
    event_id: conjure_object::Uuid,
    time: conjure_object::DateTime<conjure_object::Utc>,
}
impl BuilderStage8 {
    ///Name of the audit event, e.g. PUT_FILE
    #[inline]
    pub fn name<T>(self, name: T) -> BuilderStage9
    where
        T: Into<String>,
    {
        BuilderStage9 {
            type_: self.type_,
            deployment: self.deployment,
            host: self.host,
            product: self.product,
            product_version: self.product_version,
            producer_type: self.producer_type,
            event_id: self.event_id,
            time: self.time,
            name: name.into(),
        }
    }
}
///The stage 9 builder for the [`AuditLogV3`] type
#[derive(Debug, Clone)]
pub struct BuilderStage9 {
    type_: String,
    deployment: String,
    host: String,
    product: String,
    product_version: String,
    producer_type: super::AuditProducer,
    event_id: conjure_object::Uuid,
    time: conjure_object::DateTime<conjure_object::Utc>,
    name: String,
}
impl BuilderStage9 {
    ///Indicates whether the request was successful or the type of failure, e.g. ERROR or UNAUTHORIZED
    #[inline]
    pub fn result(self, result: super::AuditResult) -> BuilderStage10 {
        BuilderStage10 {
            type_: self.type_,
            deployment: self.deployment,
            host: self.host,
            product: self.product,
            product_version: self.product_version,
            producer_type: self.producer_type,
            event_id: self.event_id,
            time: self.time,
            name: self.name,
            result: result,
            stack: Default::default(),
            service: Default::default(),
            environment: Default::default(),
            organizations: Default::default(),
            user_agent: Default::default(),
            categories: Default::default(),
            entities: Default::default(),
            users: Default::default(),
            origins: Default::default(),
            source_origin: Default::default(),
            request_params: Default::default(),
            result_params: Default::default(),
            uid: Default::default(),
            sid: Default::default(),
            token_id: Default::default(),
            org_id: Default::default(),
            trace_id: Default::default(),
            origin: Default::default(),
        }
    }
}
///The stage 10 builder for the [`AuditLogV3`] type
#[derive(Debug, Clone)]
pub struct BuilderStage10 {
    type_: String,
    deployment: String,
    host: String,
    product: String,
    product_version: String,
    producer_type: super::AuditProducer,
    event_id: conjure_object::Uuid,
    time: conjure_object::DateTime<conjure_object::Utc>,
    name: String,
    result: super::AuditResult,
    stack: Option<String>,
    service: Option<String>,
    environment: Option<String>,
    organizations: Vec<super::Organization>,
    user_agent: Option<String>,
    categories: Vec<String>,
    entities: Vec<conjure_object::Any>,
    users: Vec<super::ContextualizedUser>,
    origins: Vec<String>,
    source_origin: Option<String>,
    request_params: std::collections::BTreeMap<String, super::SensitivityTaggedValue>,
    result_params: std::collections::BTreeMap<String, super::SensitivityTaggedValue>,
    uid: Option<super::UserId>,
    sid: Option<super::SessionId>,
    token_id: Option<super::TokenId>,
    org_id: Option<super::OrganizationId>,
    trace_id: Option<super::TraceId>,
    origin: Option<String>,
}
impl BuilderStage10 {
    ///"audit.3"
    #[inline]
    pub fn type_<T>(mut self, type_: T) -> Self
    where
        T: Into<String>,
    {
        self.type_ = type_.into();
        self
    }
    ///The deployment that produced this log. Not exposed to downstream consumers.
    #[inline]
    pub fn deployment<T>(mut self, deployment: T) -> Self
    where
        T: Into<String>,
    {
        self.deployment = deployment.into();
        self
    }
    ///The host of the service that produced this log.
    #[inline]
    pub fn host<T>(mut self, host: T) -> Self
    where
        T: Into<String>,
    {
        self.host = host.into();
        self
    }
    ///The name of the product that produced this log.
    #[inline]
    pub fn product<T>(mut self, product: T) -> Self
    where
        T: Into<String>,
    {
        self.product = product.into();
        self
    }
    ///The version of the product that produced this log.
    #[inline]
    pub fn product_version<T>(mut self, product_version: T) -> Self
    where
        T: Into<String>,
    {
        self.product_version = product_version.into();
        self
    }
    ///How this audit log was produced, eg. from a backend Server, frontend Client etc.
    #[inline]
    pub fn producer_type(mut self, producer_type: super::AuditProducer) -> Self {
        self.producer_type = producer_type;
        self
    }
    ///Unique identifier for this audit log event.
    #[inline]
    pub fn event_id(mut self, event_id: conjure_object::Uuid) -> Self {
        self.event_id = event_id;
        self
    }
    #[inline]
    pub fn time(mut self, time: conjure_object::DateTime<conjure_object::Utc>) -> Self {
        self.time = time;
        self
    }
    ///Name of the audit event, e.g. PUT_FILE
    #[inline]
    pub fn name<T>(mut self, name: T) -> Self
    where
        T: Into<String>,
    {
        self.name = name.into();
        self
    }
    ///Indicates whether the request was successful or the type of failure, e.g. ERROR or UNAUTHORIZED
    #[inline]
    pub fn result(mut self, result: super::AuditResult) -> Self {
        self.result = result;
        self
    }
    ///The stack that this log was generated on.
    #[inline]
    pub fn stack<T>(mut self, stack: T) -> Self
    where
        T: Into<Option<String>>,
    {
        self.stack = stack.into();
        self
    }
    ///The service name that produced this log.
    #[inline]
    pub fn service<T>(mut self, service: T) -> Self
    where
        T: Into<Option<String>>,
    {
        self.service = service.into();
        self
    }
    ///The environment that produced this log.
    #[inline]
    pub fn environment<T>(mut self, environment: T) -> Self
    where
        T: Into<Option<String>>,
    {
        self.environment = environment.into();
        self
    }
    ///A list of organizations that have been attributed to this log.
    ///Attribution is typically based on the user that originated this log, and the resources that
    ///they targeted.
    ///Not exposed to downstream consumers.
    #[inline]
    pub fn organizations<T>(mut self, organizations: T) -> Self
    where
        T: IntoIterator<Item = super::Organization>,
    {
        self.organizations = organizations.into_iter().collect();
        self
    }
    ///A list of organizations that have been attributed to this log.
    ///Attribution is typically based on the user that originated this log, and the resources that
    ///they targeted.
    ///Not exposed to downstream consumers.
    #[inline]
    pub fn extend_organizations<T>(mut self, organizations: T) -> Self
    where
        T: IntoIterator<Item = super::Organization>,
    {
        self.organizations.extend(organizations);
        self
    }
    ///A list of organizations that have been attributed to this log.
    ///Attribution is typically based on the user that originated this log, and the resources that
    ///they targeted.
    ///Not exposed to downstream consumers.
    #[inline]
    pub fn push_organizations(mut self, value: super::Organization) -> Self {
        self.organizations.push(value);
        self
    }
    ///The user agent of the user that originated this log.
    #[inline]
    pub fn user_agent<T>(mut self, user_agent: T) -> Self
    where
        T: Into<Option<String>>,
    {
        self.user_agent = user_agent.into();
        self
    }
    ///All audit categories produced by this audit event.
    ///Each audit categories produces a set of keys that will be distributed between the request and
    ///response params.
    #[inline]
    pub fn categories<T>(mut self, categories: T) -> Self
    where
        T: IntoIterator<Item = String>,
    {
        self.categories = categories.into_iter().collect();
        self
    }
    ///All audit categories produced by this audit event.
    ///Each audit categories produces a set of keys that will be distributed between the request and
    ///response params.
    #[inline]
    pub fn extend_categories<T>(mut self, categories: T) -> Self
    where
        T: IntoIterator<Item = String>,
    {
        self.categories.extend(categories);
        self
    }
    ///All audit categories produced by this audit event.
    ///Each audit categories produces a set of keys that will be distributed between the request and
    ///response params.
    #[inline]
    pub fn push_categories<T>(mut self, value: T) -> Self
    where
        T: Into<String>,
    {
        self.categories.push(value.into());
        self
    }
    ///All contextualized entities present in the request and response params of this log.
    ///Note: Some resources cannot be contextualized, and will not be included in this list as a result.
    #[inline]
    pub fn entities<T>(mut self, entities: T) -> Self
    where
        T: IntoIterator<Item = conjure_object::Any>,
    {
        self.entities = entities.into_iter().collect();
        self
    }
    ///All contextualized entities present in the request and response params of this log.
    ///Note: Some resources cannot be contextualized, and will not be included in this list as a result.
    #[inline]
    pub fn extend_entities<T>(mut self, entities: T) -> Self
    where
        T: IntoIterator<Item = conjure_object::Any>,
    {
        self.entities.extend(entities);
        self
    }
    ///All contextualized entities present in the request and response params of this log.
    ///Note: Some resources cannot be contextualized, and will not be included in this list as a result.
    #[inline]
    pub fn push_entities<T>(mut self, value: T) -> Self
    where
        T: conjure_object::serde::Serialize,
    {
        self.entities
            .push(conjure_object::Any::new(value).expect("value failed to serialize"));
        self
    }
    ///All contextualized users present in the request and response params of this log, including the top level
    ///UUID of this log.
    #[inline]
    pub fn users<T>(mut self, users: T) -> Self
    where
        T: IntoIterator<Item = super::ContextualizedUser>,
    {
        self.users = users.into_iter().collect();
        self
    }
    ///All contextualized users present in the request and response params of this log, including the top level
    ///UUID of this log.
    #[inline]
    pub fn extend_users<T>(mut self, users: T) -> Self
    where
        T: IntoIterator<Item = super::ContextualizedUser>,
    {
        self.users.extend(users);
        self
    }
    ///All contextualized users present in the request and response params of this log, including the top level
    ///UUID of this log.
    #[inline]
    pub fn push_users(mut self, value: super::ContextualizedUser) -> Self {
        self.users.push(value);
        self
    }
    ///All addresses attached to the request. Contains information
    ///from unreliable sources such as the X-Forwarded-For header.
    ///
    ///This value can be spoofed.
    #[inline]
    pub fn origins<T>(mut self, origins: T) -> Self
    where
        T: IntoIterator<Item = String>,
    {
        self.origins = origins.into_iter().collect();
        self
    }
    ///All addresses attached to the request. Contains information
    ///from unreliable sources such as the X-Forwarded-For header.
    ///
    ///This value can be spoofed.
    #[inline]
    pub fn extend_origins<T>(mut self, origins: T) -> Self
    where
        T: IntoIterator<Item = String>,
    {
        self.origins.extend(origins);
        self
    }
    ///All addresses attached to the request. Contains information
    ///from unreliable sources such as the X-Forwarded-For header.
    ///
    ///This value can be spoofed.
    #[inline]
    pub fn push_origins<T>(mut self, value: T) -> Self
    where
        T: Into<String>,
    {
        self.origins.push(value.into());
        self
    }
    ///Origin of the network request. If a request goes through a proxy,
    ///this will contain the proxy''s address.
    ///
    ///This value is verified through the TCP stack.
    #[inline]
    pub fn source_origin<T>(mut self, source_origin: T) -> Self
    where
        T: Into<Option<String>>,
    {
        self.source_origin = source_origin.into();
        self
    }
    ///The parameters known at method invocation time.
    ///
    ///Note that all keys must be known to the audit library. Typically, entries in the request and response
    ///params will be dependent on the `categories` field defined above.
    #[inline]
    pub fn request_params<T>(mut self, request_params: T) -> Self
    where
        T: IntoIterator<Item = (String, super::SensitivityTaggedValue)>,
    {
        self.request_params = request_params.into_iter().collect();
        self
    }
    ///The parameters known at method invocation time.
    ///
    ///Note that all keys must be known to the audit library. Typically, entries in the request and response
    ///params will be dependent on the `categories` field defined above.
    #[inline]
    pub fn extend_request_params<T>(mut self, request_params: T) -> Self
    where
        T: IntoIterator<Item = (String, super::SensitivityTaggedValue)>,
    {
        self.request_params.extend(request_params);
        self
    }
    ///The parameters known at method invocation time.
    ///
    ///Note that all keys must be known to the audit library. Typically, entries in the request and response
    ///params will be dependent on the `categories` field defined above.
    #[inline]
    pub fn insert_request_params<K>(
        mut self,
        key: K,
        value: super::SensitivityTaggedValue,
    ) -> Self
    where
        K: Into<String>,
    {
        self.request_params.insert(key.into(), value);
        self
    }
    ///Information derived within a method, commonly parts of the return value.
    ///
    ///Note that all keys must be known to the audit library. Typically, entries in the request and response
    ///params will be dependent on the `categories` field defined above.
    #[inline]
    pub fn result_params<T>(mut self, result_params: T) -> Self
    where
        T: IntoIterator<Item = (String, super::SensitivityTaggedValue)>,
    {
        self.result_params = result_params.into_iter().collect();
        self
    }
    ///Information derived within a method, commonly parts of the return value.
    ///
    ///Note that all keys must be known to the audit library. Typically, entries in the request and response
    ///params will be dependent on the `categories` field defined above.
    #[inline]
    pub fn extend_result_params<T>(mut self, result_params: T) -> Self
    where
        T: IntoIterator<Item = (String, super::SensitivityTaggedValue)>,
    {
        self.result_params.extend(result_params);
        self
    }
    ///Information derived within a method, commonly parts of the return value.
    ///
    ///Note that all keys must be known to the audit library. Typically, entries in the request and response
    ///params will be dependent on the `categories` field defined above.
    #[inline]
    pub fn insert_result_params<K>(
        mut self,
        key: K,
        value: super::SensitivityTaggedValue,
    ) -> Self
    where
        K: Into<String>,
    {
        self.result_params.insert(key.into(), value);
        self
    }
    ///User id (if available). This is the most downstream caller.
    #[inline]
    pub fn uid<T>(mut self, uid: T) -> Self
    where
        T: Into<Option<super::UserId>>,
    {
        self.uid = uid.into();
        self
    }
    ///Session id (if available)
    #[inline]
    pub fn sid<T>(mut self, sid: T) -> Self
    where
        T: Into<Option<super::SessionId>>,
    {
        self.sid = sid.into();
        self
    }
    ///API token id (if available)
    #[inline]
    pub fn token_id<T>(mut self, token_id: T) -> Self
    where
        T: Into<Option<super::TokenId>>,
    {
        self.token_id = token_id.into();
        self
    }
    ///Organization id (if available)
    #[inline]
    pub fn org_id<T>(mut self, org_id: T) -> Self
    where
        T: Into<Option<super::OrganizationId>>,
    {
        self.org_id = org_id.into();
        self
    }
    ///Zipkin trace id (if available)
    #[inline]
    pub fn trace_id<T>(mut self, trace_id: T) -> Self
    where
        T: Into<Option<super::TraceId>>,
    {
        self.trace_id = trace_id.into();
        self
    }
    ///Best-effort identifier of the originating machine, e.g. an
    ///IP address, a Kubernetes node identifier, or similar.
    ///
    ///This value can be spoofed.
    #[inline]
    pub fn origin<T>(mut self, origin: T) -> Self
    where
        T: Into<Option<String>>,
    {
        self.origin = origin.into();
        self
    }
    /// Consumes the builder, constructing a new instance of the type.
    #[inline]
    pub fn build(self) -> AuditLogV3 {
        AuditLogV3 {
            type_: self.type_,
            deployment: self.deployment,
            host: self.host,
            product: self.product,
            product_version: self.product_version,
            stack: self.stack,
            service: self.service,
            environment: self.environment,
            producer_type: self.producer_type,
            organizations: self.organizations,
            event_id: self.event_id,
            user_agent: self.user_agent,
            categories: self.categories,
            entities: self.entities,
            users: self.users,
            origins: self.origins,
            source_origin: self.source_origin,
            request_params: self.request_params,
            result_params: self.result_params,
            time: self.time,
            uid: self.uid,
            sid: self.sid,
            token_id: self.token_id,
            org_id: self.org_id,
            trace_id: self.trace_id,
            origin: self.origin,
            name: self.name,
            result: self.result,
        }
    }
}
impl ser::Serialize for AuditLogV3 {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut size = 10usize;
        let skip_stack = self.stack.is_none();
        if !skip_stack {
            size += 1;
        }
        let skip_service = self.service.is_none();
        if !skip_service {
            size += 1;
        }
        let skip_environment = self.environment.is_none();
        if !skip_environment {
            size += 1;
        }
        let skip_organizations = self.organizations.is_empty();
        if !skip_organizations {
            size += 1;
        }
        let skip_user_agent = self.user_agent.is_none();
        if !skip_user_agent {
            size += 1;
        }
        let skip_categories = self.categories.is_empty();
        if !skip_categories {
            size += 1;
        }
        let skip_entities = self.entities.is_empty();
        if !skip_entities {
            size += 1;
        }
        let skip_users = self.users.is_empty();
        if !skip_users {
            size += 1;
        }
        let skip_origins = self.origins.is_empty();
        if !skip_origins {
            size += 1;
        }
        let skip_source_origin = self.source_origin.is_none();
        if !skip_source_origin {
            size += 1;
        }
        let skip_request_params = self.request_params.is_empty();
        if !skip_request_params {
            size += 1;
        }
        let skip_result_params = self.result_params.is_empty();
        if !skip_result_params {
            size += 1;
        }
        let skip_uid = self.uid.is_none();
        if !skip_uid {
            size += 1;
        }
        let skip_sid = self.sid.is_none();
        if !skip_sid {
            size += 1;
        }
        let skip_token_id = self.token_id.is_none();
        if !skip_token_id {
            size += 1;
        }
        let skip_org_id = self.org_id.is_none();
        if !skip_org_id {
            size += 1;
        }
        let skip_trace_id = self.trace_id.is_none();
        if !skip_trace_id {
            size += 1;
        }
        let skip_origin = self.origin.is_none();
        if !skip_origin {
            size += 1;
        }
        let mut s = s.serialize_struct("AuditLogV3", size)?;
        s.serialize_field("type", &self.type_)?;
        s.serialize_field("deployment", &self.deployment)?;
        s.serialize_field("host", &self.host)?;
        s.serialize_field("product", &self.product)?;
        s.serialize_field("productVersion", &self.product_version)?;
        if skip_stack {
            s.skip_field("stack")?;
        } else {
            s.serialize_field("stack", &self.stack)?;
        }
        if skip_service {
            s.skip_field("service")?;
        } else {
            s.serialize_field("service", &self.service)?;
        }
        if skip_environment {
            s.skip_field("environment")?;
        } else {
            s.serialize_field("environment", &self.environment)?;
        }
        s.serialize_field("producerType", &self.producer_type)?;
        if skip_organizations {
            s.skip_field("organizations")?;
        } else {
            s.serialize_field("organizations", &self.organizations)?;
        }
        s.serialize_field("eventId", &self.event_id)?;
        if skip_user_agent {
            s.skip_field("userAgent")?;
        } else {
            s.serialize_field("userAgent", &self.user_agent)?;
        }
        if skip_categories {
            s.skip_field("categories")?;
        } else {
            s.serialize_field("categories", &self.categories)?;
        }
        if skip_entities {
            s.skip_field("entities")?;
        } else {
            s.serialize_field("entities", &self.entities)?;
        }
        if skip_users {
            s.skip_field("users")?;
        } else {
            s.serialize_field("users", &self.users)?;
        }
        if skip_origins {
            s.skip_field("origins")?;
        } else {
            s.serialize_field("origins", &self.origins)?;
        }
        if skip_source_origin {
            s.skip_field("sourceOrigin")?;
        } else {
            s.serialize_field("sourceOrigin", &self.source_origin)?;
        }
        if skip_request_params {
            s.skip_field("requestParams")?;
        } else {
            s.serialize_field("requestParams", &self.request_params)?;
        }
        if skip_result_params {
            s.skip_field("resultParams")?;
        } else {
            s.serialize_field("resultParams", &self.result_params)?;
        }
        s.serialize_field("time", &self.time)?;
        if skip_uid {
            s.skip_field("uid")?;
        } else {
            s.serialize_field("uid", &self.uid)?;
        }
        if skip_sid {
            s.skip_field("sid")?;
        } else {
            s.serialize_field("sid", &self.sid)?;
        }
        if skip_token_id {
            s.skip_field("tokenId")?;
        } else {
            s.serialize_field("tokenId", &self.token_id)?;
        }
        if skip_org_id {
            s.skip_field("orgId")?;
        } else {
            s.serialize_field("orgId", &self.org_id)?;
        }
        if skip_trace_id {
            s.skip_field("traceId")?;
        } else {
            s.serialize_field("traceId", &self.trace_id)?;
        }
        if skip_origin {
            s.skip_field("origin")?;
        } else {
            s.serialize_field("origin", &self.origin)?;
        }
        s.serialize_field("name", &self.name)?;
        s.serialize_field("result", &self.result)?;
        s.end()
    }
}
impl<'de> de::Deserialize<'de> for AuditLogV3 {
    fn deserialize<D>(d: D) -> Result<AuditLogV3, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_struct(
            "AuditLogV3",
            &[
                "type",
                "deployment",
                "host",
                "product",
                "productVersion",
                "stack",
                "service",
                "environment",
                "producerType",
                "organizations",
                "eventId",
                "userAgent",
                "categories",
                "entities",
                "users",
                "origins",
                "sourceOrigin",
                "requestParams",
                "resultParams",
                "time",
                "uid",
                "sid",
                "tokenId",
                "orgId",
                "traceId",
                "origin",
                "name",
                "result",
            ],
            Visitor_,
        )
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = AuditLogV3;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("map")
    }
    fn visit_map<A>(self, mut map_: A) -> Result<AuditLogV3, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut type_ = None;
        let mut deployment = None;
        let mut host = None;
        let mut product = None;
        let mut product_version = None;
        let mut stack = None;
        let mut service = None;
        let mut environment = None;
        let mut producer_type = None;
        let mut organizations = None;
        let mut event_id = None;
        let mut user_agent = None;
        let mut categories = None;
        let mut entities = None;
        let mut users = None;
        let mut origins = None;
        let mut source_origin = None;
        let mut request_params = None;
        let mut result_params = None;
        let mut time = None;
        let mut uid = None;
        let mut sid = None;
        let mut token_id = None;
        let mut org_id = None;
        let mut trace_id = None;
        let mut origin = None;
        let mut name = None;
        let mut result = None;
        while let Some(field_) = map_.next_key()? {
            match field_ {
                Field_::Type => type_ = Some(map_.next_value()?),
                Field_::Deployment => deployment = Some(map_.next_value()?),
                Field_::Host => host = Some(map_.next_value()?),
                Field_::Product => product = Some(map_.next_value()?),
                Field_::ProductVersion => product_version = Some(map_.next_value()?),
                Field_::Stack => stack = Some(map_.next_value()?),
                Field_::Service => service = Some(map_.next_value()?),
                Field_::Environment => environment = Some(map_.next_value()?),
                Field_::ProducerType => producer_type = Some(map_.next_value()?),
                Field_::Organizations => organizations = Some(map_.next_value()?),
                Field_::EventId => event_id = Some(map_.next_value()?),
                Field_::UserAgent => user_agent = Some(map_.next_value()?),
                Field_::Categories => categories = Some(map_.next_value()?),
                Field_::Entities => entities = Some(map_.next_value()?),
                Field_::Users => users = Some(map_.next_value()?),
                Field_::Origins => origins = Some(map_.next_value()?),
                Field_::SourceOrigin => source_origin = Some(map_.next_value()?),
                Field_::RequestParams => request_params = Some(map_.next_value()?),
                Field_::ResultParams => result_params = Some(map_.next_value()?),
                Field_::Time => time = Some(map_.next_value()?),
                Field_::Uid => uid = Some(map_.next_value()?),
                Field_::Sid => sid = Some(map_.next_value()?),
                Field_::TokenId => token_id = Some(map_.next_value()?),
                Field_::OrgId => org_id = Some(map_.next_value()?),
                Field_::TraceId => trace_id = Some(map_.next_value()?),
                Field_::Origin => origin = Some(map_.next_value()?),
                Field_::Name => name = Some(map_.next_value()?),
                Field_::Result => result = Some(map_.next_value()?),
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
        let host = match host {
            Some(v) => v,
            None => return Err(de::Error::missing_field("host")),
        };
        let product = match product {
            Some(v) => v,
            None => return Err(de::Error::missing_field("product")),
        };
        let product_version = match product_version {
            Some(v) => v,
            None => return Err(de::Error::missing_field("productVersion")),
        };
        let stack = match stack {
            Some(v) => v,
            None => Default::default(),
        };
        let service = match service {
            Some(v) => v,
            None => Default::default(),
        };
        let environment = match environment {
            Some(v) => v,
            None => Default::default(),
        };
        let producer_type = match producer_type {
            Some(v) => v,
            None => return Err(de::Error::missing_field("producerType")),
        };
        let organizations = match organizations {
            Some(v) => v,
            None => Default::default(),
        };
        let event_id = match event_id {
            Some(v) => v,
            None => return Err(de::Error::missing_field("eventId")),
        };
        let user_agent = match user_agent {
            Some(v) => v,
            None => Default::default(),
        };
        let categories = match categories {
            Some(v) => v,
            None => Default::default(),
        };
        let entities = match entities {
            Some(v) => v,
            None => Default::default(),
        };
        let users = match users {
            Some(v) => v,
            None => Default::default(),
        };
        let origins = match origins {
            Some(v) => v,
            None => Default::default(),
        };
        let source_origin = match source_origin {
            Some(v) => v,
            None => Default::default(),
        };
        let request_params = match request_params {
            Some(v) => v,
            None => Default::default(),
        };
        let result_params = match result_params {
            Some(v) => v,
            None => Default::default(),
        };
        let time = match time {
            Some(v) => v,
            None => return Err(de::Error::missing_field("time")),
        };
        let uid = match uid {
            Some(v) => v,
            None => Default::default(),
        };
        let sid = match sid {
            Some(v) => v,
            None => Default::default(),
        };
        let token_id = match token_id {
            Some(v) => v,
            None => Default::default(),
        };
        let org_id = match org_id {
            Some(v) => v,
            None => Default::default(),
        };
        let trace_id = match trace_id {
            Some(v) => v,
            None => Default::default(),
        };
        let origin = match origin {
            Some(v) => v,
            None => Default::default(),
        };
        let name = match name {
            Some(v) => v,
            None => return Err(de::Error::missing_field("name")),
        };
        let result = match result {
            Some(v) => v,
            None => return Err(de::Error::missing_field("result")),
        };
        Ok(AuditLogV3 {
            type_,
            deployment,
            host,
            product,
            product_version,
            stack,
            service,
            environment,
            producer_type,
            organizations,
            event_id,
            user_agent,
            categories,
            entities,
            users,
            origins,
            source_origin,
            request_params,
            result_params,
            time,
            uid,
            sid,
            token_id,
            org_id,
            trace_id,
            origin,
            name,
            result,
        })
    }
}
enum Field_ {
    Type,
    Deployment,
    Host,
    Product,
    ProductVersion,
    Stack,
    Service,
    Environment,
    ProducerType,
    Organizations,
    EventId,
    UserAgent,
    Categories,
    Entities,
    Users,
    Origins,
    SourceOrigin,
    RequestParams,
    ResultParams,
    Time,
    Uid,
    Sid,
    TokenId,
    OrgId,
    TraceId,
    Origin,
    Name,
    Result,
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
            "host" => Field_::Host,
            "product" => Field_::Product,
            "productVersion" => Field_::ProductVersion,
            "stack" => Field_::Stack,
            "service" => Field_::Service,
            "environment" => Field_::Environment,
            "producerType" => Field_::ProducerType,
            "organizations" => Field_::Organizations,
            "eventId" => Field_::EventId,
            "userAgent" => Field_::UserAgent,
            "categories" => Field_::Categories,
            "entities" => Field_::Entities,
            "users" => Field_::Users,
            "origins" => Field_::Origins,
            "sourceOrigin" => Field_::SourceOrigin,
            "requestParams" => Field_::RequestParams,
            "resultParams" => Field_::ResultParams,
            "time" => Field_::Time,
            "uid" => Field_::Uid,
            "sid" => Field_::Sid,
            "tokenId" => Field_::TokenId,
            "orgId" => Field_::OrgId,
            "traceId" => Field_::TraceId,
            "origin" => Field_::Origin,
            "name" => Field_::Name,
            "result" => Field_::Result,
            _ => Field_::Unknown_,
        };
        Ok(v)
    }
}
