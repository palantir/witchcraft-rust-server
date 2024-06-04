use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[conjure_object::private::staged_builder::staged_builder]
#[builder(crate = conjure_object::private::staged_builder, update, inline)]
pub struct AuditLogV3 {
    #[builder(into)]
    type_: String,
    #[builder(into)]
    deployment: String,
    #[builder(into)]
    host: String,
    #[builder(into)]
    product: String,
    #[builder(into)]
    product_version: String,
    #[builder(default, into)]
    stack: Option<String>,
    #[builder(default, into)]
    service: Option<String>,
    #[builder(default, into)]
    environment: Option<String>,
    producer_type: super::AuditProducer,
    #[builder(default, list(item(type = super::Organization)))]
    organizations: Vec<super::Organization>,
    event_id: conjure_object::Uuid,
    #[builder(default, into)]
    user_agent: Option<String>,
    #[builder(default, list(item(type = String, into)))]
    categories: Vec<String>,
    #[builder(
        default,
        list(
            item(
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
    entities: Vec<conjure_object::Any>,
    #[builder(default, list(item(type = super::ContextualizedUser)))]
    users: Vec<super::ContextualizedUser>,
    #[builder(default, list(item(type = String, into)))]
    origins: Vec<String>,
    #[builder(default, into)]
    source_origin: Option<String>,
    #[builder(
        default,
        map(key(type = String, into), value(type = super::SensitivityTaggedValue))
    )]
    request_params: std::collections::BTreeMap<String, super::SensitivityTaggedValue>,
    #[builder(
        default,
        map(key(type = String, into), value(type = super::SensitivityTaggedValue))
    )]
    result_params: std::collections::BTreeMap<String, super::SensitivityTaggedValue>,
    time: conjure_object::DateTime<conjure_object::Utc>,
    #[builder(default, into)]
    uid: Option<super::UserId>,
    #[builder(default, into)]
    sid: Option<super::SessionId>,
    #[builder(default, into)]
    token_id: Option<super::TokenId>,
    #[builder(default, into)]
    org_id: Option<super::OrganizationId>,
    #[builder(default, into)]
    trace_id: Option<super::TraceId>,
    #[builder(default, into)]
    origin: Option<String>,
    #[builder(into)]
    name: String,
    result: super::AuditResult,
}
impl AuditLogV3 {
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
