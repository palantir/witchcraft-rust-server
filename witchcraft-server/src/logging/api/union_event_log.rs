use conjure_object::private::{UnionField_, UnionTypeField_};
use conjure_object::serde::ser::SerializeMap as SerializeMap_;
use conjure_object::serde::{de, ser};
use std::fmt;
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum UnionEventLog {
    EventLog(super::EventLogV1),
    EventLogV2(super::EventLogV2),
}
impl ser::Serialize for UnionEventLog {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut map = s.serialize_map(Some(2))?;
        match self {
            UnionEventLog::EventLog(value) => {
                map.serialize_entry(&"type", &"eventLog")?;
                map.serialize_entry(&"eventLog", value)?;
            }
            UnionEventLog::EventLogV2(value) => {
                map.serialize_entry(&"type", &"eventLogV2")?;
                map.serialize_entry(&"eventLogV2", value)?;
            }
        }
        map.end()
    }
}
impl<'de> de::Deserialize<'de> for UnionEventLog {
    fn deserialize<D>(d: D) -> Result<UnionEventLog, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_map(Visitor_)
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = UnionEventLog;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("union UnionEventLog")
    }
    fn visit_map<A>(self, mut map: A) -> Result<UnionEventLog, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let v = match map.next_key::<UnionField_<Variant_>>()? {
            Some(UnionField_::Type) => {
                let variant = map.next_value()?;
                let key = map.next_key()?;
                match (variant, key) {
                    (Variant_::EventLog, Some(Variant_::EventLog)) => {
                        let value = map.next_value()?;
                        UnionEventLog::EventLog(value)
                    }
                    (Variant_::EventLogV2, Some(Variant_::EventLogV2)) => {
                        let value = map.next_value()?;
                        UnionEventLog::EventLogV2(value)
                    }
                    (variant, Some(key)) => {
                        return Err(de::Error::invalid_value(
                            de::Unexpected::Str(key.as_str()),
                            &variant.as_str(),
                        ));
                    }
                    (variant, None) => return Err(de::Error::missing_field(variant.as_str())),
                }
            }
            Some(UnionField_::Value(variant)) => {
                let value = match &variant {
                    Variant_::EventLog => {
                        let value = map.next_value()?;
                        UnionEventLog::EventLog(value)
                    }
                    Variant_::EventLogV2 => {
                        let value = map.next_value()?;
                        UnionEventLog::EventLogV2(value)
                    }
                };
                if map.next_key::<UnionTypeField_>()?.is_none() {
                    return Err(de::Error::missing_field("type"));
                }
                let type_variant = map.next_value::<Variant_>()?;
                if variant != type_variant {
                    return Err(de::Error::invalid_value(
                        de::Unexpected::Str(type_variant.as_str()),
                        &variant.as_str(),
                    ));
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
    EventLog,
    EventLogV2,
}
impl Variant_ {
    fn as_str(&self) -> &'static str {
        match self {
            Variant_::EventLog => "eventLog",
            Variant_::EventLogV2 => "eventLogV2",
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
            "eventLog" => Variant_::EventLog,
            "eventLogV2" => Variant_::EventLogV2,
            value => {
                return Err(de::Error::unknown_variant(
                    value,
                    &["eventLog", "eventLogV2"],
                ))
            }
        };
        Ok(v)
    }
}
