use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
///Wraps a log entry with entity information.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[conjure_object::private::staged_builder::staged_builder]
#[builder(crate = conjure_object::private::staged_builder, update, inline)]
pub struct WrappedLogV1 {
    #[builder(into)]
    type_: String,
    #[builder(custom(type = super::WrappedLogV1Payload, convert = Box::new))]
    payload: Box<super::WrappedLogV1Payload>,
    #[builder(into)]
    entity_name: String,
    #[builder(into)]
    entity_version: String,
    #[builder(default, into)]
    service: Option<String>,
    #[builder(default, into)]
    service_id: Option<String>,
    #[builder(default, into)]
    stack: Option<String>,
    #[builder(default, into)]
    stack_id: Option<String>,
}
impl WrappedLogV1 {
    ///"wrapped.1"
    #[inline]
    pub fn type_(&self) -> &str {
        &*self.type_
    }
    #[inline]
    pub fn payload(&self) -> &super::WrappedLogV1Payload {
        &*self.payload
    }
    ///Artifact part of entity's maven coordinate
    #[inline]
    pub fn entity_name(&self) -> &str {
        &*self.entity_name
    }
    #[inline]
    pub fn entity_version(&self) -> &str {
        &*self.entity_version
    }
    ///Defaults to the wrapped log producer's Skylab service name.
    #[inline]
    pub fn service(&self) -> Option<&str> {
        self.service.as_ref().map(|o| &**o)
    }
    ///Defaults to the wrapped log producer's Skylab service ID.
    #[inline]
    pub fn service_id(&self) -> Option<&str> {
        self.service_id.as_ref().map(|o| &**o)
    }
    ///Defaults to the wrapped log producer's Skylab stack name.
    #[inline]
    pub fn stack(&self) -> Option<&str> {
        self.stack.as_ref().map(|o| &**o)
    }
    ///Defaults to the wrapped log producer's Skylab stack ID.
    #[inline]
    pub fn stack_id(&self) -> Option<&str> {
        self.stack_id.as_ref().map(|o| &**o)
    }
}
impl ser::Serialize for WrappedLogV1 {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut size = 4usize;
        let skip_service = self.service.is_none();
        if !skip_service {
            size += 1;
        }
        let skip_service_id = self.service_id.is_none();
        if !skip_service_id {
            size += 1;
        }
        let skip_stack = self.stack.is_none();
        if !skip_stack {
            size += 1;
        }
        let skip_stack_id = self.stack_id.is_none();
        if !skip_stack_id {
            size += 1;
        }
        let mut s = s.serialize_struct("WrappedLogV1", size)?;
        s.serialize_field("type", &self.type_)?;
        s.serialize_field("payload", &self.payload)?;
        s.serialize_field("entityName", &self.entity_name)?;
        s.serialize_field("entityVersion", &self.entity_version)?;
        if skip_service {
            s.skip_field("service")?;
        } else {
            s.serialize_field("service", &self.service)?;
        }
        if skip_service_id {
            s.skip_field("serviceId")?;
        } else {
            s.serialize_field("serviceId", &self.service_id)?;
        }
        if skip_stack {
            s.skip_field("stack")?;
        } else {
            s.serialize_field("stack", &self.stack)?;
        }
        if skip_stack_id {
            s.skip_field("stackId")?;
        } else {
            s.serialize_field("stackId", &self.stack_id)?;
        }
        s.end()
    }
}
impl<'de> de::Deserialize<'de> for WrappedLogV1 {
    fn deserialize<D>(d: D) -> Result<WrappedLogV1, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_struct(
            "WrappedLogV1",
            &[
                "type",
                "payload",
                "entityName",
                "entityVersion",
                "service",
                "serviceId",
                "stack",
                "stackId",
            ],
            Visitor_,
        )
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = WrappedLogV1;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("map")
    }
    fn visit_map<A>(self, mut map_: A) -> Result<WrappedLogV1, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut type_ = None;
        let mut payload = None;
        let mut entity_name = None;
        let mut entity_version = None;
        let mut service = None;
        let mut service_id = None;
        let mut stack = None;
        let mut stack_id = None;
        while let Some(field_) = map_.next_key()? {
            match field_ {
                Field_::Type => type_ = Some(map_.next_value()?),
                Field_::Payload => payload = Some(map_.next_value()?),
                Field_::EntityName => entity_name = Some(map_.next_value()?),
                Field_::EntityVersion => entity_version = Some(map_.next_value()?),
                Field_::Service => service = Some(map_.next_value()?),
                Field_::ServiceId => service_id = Some(map_.next_value()?),
                Field_::Stack => stack = Some(map_.next_value()?),
                Field_::StackId => stack_id = Some(map_.next_value()?),
                Field_::Unknown_ => {
                    map_.next_value::<de::IgnoredAny>()?;
                }
            }
        }
        let type_ = match type_ {
            Some(v) => v,
            None => return Err(de::Error::missing_field("type")),
        };
        let payload = match payload {
            Some(v) => v,
            None => return Err(de::Error::missing_field("payload")),
        };
        let entity_name = match entity_name {
            Some(v) => v,
            None => return Err(de::Error::missing_field("entityName")),
        };
        let entity_version = match entity_version {
            Some(v) => v,
            None => return Err(de::Error::missing_field("entityVersion")),
        };
        let service = match service {
            Some(v) => v,
            None => Default::default(),
        };
        let service_id = match service_id {
            Some(v) => v,
            None => Default::default(),
        };
        let stack = match stack {
            Some(v) => v,
            None => Default::default(),
        };
        let stack_id = match stack_id {
            Some(v) => v,
            None => Default::default(),
        };
        Ok(WrappedLogV1 {
            type_,
            payload,
            entity_name,
            entity_version,
            service,
            service_id,
            stack,
            stack_id,
        })
    }
}
enum Field_ {
    Type,
    Payload,
    EntityName,
    EntityVersion,
    Service,
    ServiceId,
    Stack,
    StackId,
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
            "payload" => Field_::Payload,
            "entityName" => Field_::EntityName,
            "entityVersion" => Field_::EntityVersion,
            "service" => Field_::Service,
            "serviceId" => Field_::ServiceId,
            "stack" => Field_::Stack,
            "stackId" => Field_::StackId,
            _ => Field_::Unknown_,
        };
        Ok(v)
    }
}
