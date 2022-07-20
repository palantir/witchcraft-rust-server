use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeMap as SerializeMap_;
use conjure_object::private::{UnionField_, UnionTypeField_};
use std::fmt;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WrappedLogV1Payload {
    ServiceLogV1(super::ServiceLogV1),
    RequestLogV2(super::RequestLogV2),
    TraceLogV1(super::TraceLogV1),
    EventLogV2(super::EventLogV2),
    MetricLogV1(super::MetricLogV1),
    AuditLogV2(super::AuditLogV2),
    DiagnosticLogV1(super::DiagnosticLogV1),
}
impl ser::Serialize for WrappedLogV1Payload {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut map = s.serialize_map(Some(2))?;
        match self {
            WrappedLogV1Payload::ServiceLogV1(value) => {
                map.serialize_entry(&"type", &"serviceLogV1")?;
                map.serialize_entry(&"serviceLogV1", value)?;
            }
            WrappedLogV1Payload::RequestLogV2(value) => {
                map.serialize_entry(&"type", &"requestLogV2")?;
                map.serialize_entry(&"requestLogV2", value)?;
            }
            WrappedLogV1Payload::TraceLogV1(value) => {
                map.serialize_entry(&"type", &"traceLogV1")?;
                map.serialize_entry(&"traceLogV1", value)?;
            }
            WrappedLogV1Payload::EventLogV2(value) => {
                map.serialize_entry(&"type", &"eventLogV2")?;
                map.serialize_entry(&"eventLogV2", value)?;
            }
            WrappedLogV1Payload::MetricLogV1(value) => {
                map.serialize_entry(&"type", &"metricLogV1")?;
                map.serialize_entry(&"metricLogV1", value)?;
            }
            WrappedLogV1Payload::AuditLogV2(value) => {
                map.serialize_entry(&"type", &"auditLogV2")?;
                map.serialize_entry(&"auditLogV2", value)?;
            }
            WrappedLogV1Payload::DiagnosticLogV1(value) => {
                map.serialize_entry(&"type", &"diagnosticLogV1")?;
                map.serialize_entry(&"diagnosticLogV1", value)?;
            }
        }
        map.end()
    }
}
impl<'de> de::Deserialize<'de> for WrappedLogV1Payload {
    fn deserialize<D>(d: D) -> Result<WrappedLogV1Payload, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_map(Visitor_)
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = WrappedLogV1Payload;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("union WrappedLogV1Payload")
    }
    fn visit_map<A>(self, mut map: A) -> Result<WrappedLogV1Payload, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let v = match map.next_key::<UnionField_<Variant_>>()? {
            Some(UnionField_::Type) => {
                let variant = map.next_value()?;
                let key = map.next_key()?;
                match (variant, key) {
                    (Variant_::ServiceLogV1, Some(Variant_::ServiceLogV1)) => {
                        let value = map.next_value()?;
                        WrappedLogV1Payload::ServiceLogV1(value)
                    }
                    (Variant_::RequestLogV2, Some(Variant_::RequestLogV2)) => {
                        let value = map.next_value()?;
                        WrappedLogV1Payload::RequestLogV2(value)
                    }
                    (Variant_::TraceLogV1, Some(Variant_::TraceLogV1)) => {
                        let value = map.next_value()?;
                        WrappedLogV1Payload::TraceLogV1(value)
                    }
                    (Variant_::EventLogV2, Some(Variant_::EventLogV2)) => {
                        let value = map.next_value()?;
                        WrappedLogV1Payload::EventLogV2(value)
                    }
                    (Variant_::MetricLogV1, Some(Variant_::MetricLogV1)) => {
                        let value = map.next_value()?;
                        WrappedLogV1Payload::MetricLogV1(value)
                    }
                    (Variant_::AuditLogV2, Some(Variant_::AuditLogV2)) => {
                        let value = map.next_value()?;
                        WrappedLogV1Payload::AuditLogV2(value)
                    }
                    (Variant_::DiagnosticLogV1, Some(Variant_::DiagnosticLogV1)) => {
                        let value = map.next_value()?;
                        WrappedLogV1Payload::DiagnosticLogV1(value)
                    }
                    (variant, Some(key)) => {
                        return Err(
                            de::Error::invalid_value(
                                de::Unexpected::Str(key.as_str()),
                                &variant.as_str(),
                            ),
                        );
                    }
                    (variant, None) => {
                        return Err(de::Error::missing_field(variant.as_str()));
                    }
                }
            }
            Some(UnionField_::Value(variant)) => {
                let value = match &variant {
                    Variant_::ServiceLogV1 => {
                        let value = map.next_value()?;
                        WrappedLogV1Payload::ServiceLogV1(value)
                    }
                    Variant_::RequestLogV2 => {
                        let value = map.next_value()?;
                        WrappedLogV1Payload::RequestLogV2(value)
                    }
                    Variant_::TraceLogV1 => {
                        let value = map.next_value()?;
                        WrappedLogV1Payload::TraceLogV1(value)
                    }
                    Variant_::EventLogV2 => {
                        let value = map.next_value()?;
                        WrappedLogV1Payload::EventLogV2(value)
                    }
                    Variant_::MetricLogV1 => {
                        let value = map.next_value()?;
                        WrappedLogV1Payload::MetricLogV1(value)
                    }
                    Variant_::AuditLogV2 => {
                        let value = map.next_value()?;
                        WrappedLogV1Payload::AuditLogV2(value)
                    }
                    Variant_::DiagnosticLogV1 => {
                        let value = map.next_value()?;
                        WrappedLogV1Payload::DiagnosticLogV1(value)
                    }
                };
                if map.next_key::<UnionTypeField_>()?.is_none() {
                    return Err(de::Error::missing_field("type"));
                }
                let type_variant = map.next_value::<Variant_>()?;
                if variant != type_variant {
                    return Err(
                        de::Error::invalid_value(
                            de::Unexpected::Str(type_variant.as_str()),
                            &variant.as_str(),
                        ),
                    );
                }
                value
            }
            None => return Err(de::Error::missing_field("type")),
        };
        if map.next_key::<UnionField_<Variant_>>()?.is_some() {
            return Err(de::Error::invalid_length(3, &"type and value fields"));
        }
        Ok(v)
    }
}
#[derive(PartialEq)]
enum Variant_ {
    ServiceLogV1,
    RequestLogV2,
    TraceLogV1,
    EventLogV2,
    MetricLogV1,
    AuditLogV2,
    DiagnosticLogV1,
}
impl Variant_ {
    fn as_str(&self) -> &'static str {
        match self {
            Variant_::ServiceLogV1 => "serviceLogV1",
            Variant_::RequestLogV2 => "requestLogV2",
            Variant_::TraceLogV1 => "traceLogV1",
            Variant_::EventLogV2 => "eventLogV2",
            Variant_::MetricLogV1 => "metricLogV1",
            Variant_::AuditLogV2 => "auditLogV2",
            Variant_::DiagnosticLogV1 => "diagnosticLogV1",
        }
    }
}
impl<'de> de::Deserialize<'de> for Variant_ {
    fn deserialize<D>(d: D) -> Result<Variant_, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_str(VariantVisitor_)
    }
}
struct VariantVisitor_;
impl<'de> de::Visitor<'de> for VariantVisitor_ {
    type Value = Variant_;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("string")
    }
    fn visit_str<E>(self, value: &str) -> Result<Variant_, E>
    where
        E: de::Error,
    {
        let v = match value {
            "serviceLogV1" => Variant_::ServiceLogV1,
            "requestLogV2" => Variant_::RequestLogV2,
            "traceLogV1" => Variant_::TraceLogV1,
            "eventLogV2" => Variant_::EventLogV2,
            "metricLogV1" => Variant_::MetricLogV1,
            "auditLogV2" => Variant_::AuditLogV2,
            "diagnosticLogV1" => Variant_::DiagnosticLogV1,
            value => {
                return Err(
                    de::Error::unknown_variant(
                        value,
                        &[
                            "serviceLogV1",
                            "requestLogV2",
                            "traceLogV1",
                            "eventLogV2",
                            "metricLogV1",
                            "auditLogV2",
                            "diagnosticLogV1",
                        ],
                    ),
                );
            }
        };
        Ok(v)
    }
}
