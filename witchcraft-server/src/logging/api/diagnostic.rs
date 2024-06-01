use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeMap as SerializeMap_;
use conjure_object::private::{UnionField_, UnionTypeField_};
use std::fmt;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Diagnostic {
    Generic(super::GenericDiagnostic),
    ThreadDump(super::ThreadDumpV1),
}
impl ser::Serialize for Diagnostic {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut map = s.serialize_map(Some(2))?;
        match self {
            Diagnostic::Generic(value) => {
                map.serialize_entry(&"type", &"generic")?;
                map.serialize_entry(&"generic", value)?;
            }
            Diagnostic::ThreadDump(value) => {
                map.serialize_entry(&"type", &"threadDump")?;
                map.serialize_entry(&"threadDump", value)?;
            }
        }
        map.end()
    }
}
impl<'de> de::Deserialize<'de> for Diagnostic {
    fn deserialize<D>(d: D) -> Result<Diagnostic, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_map(Visitor_)
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = Diagnostic;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("union Diagnostic")
    }
    fn visit_map<A>(self, mut map: A) -> Result<Diagnostic, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let v = match map.next_key::<UnionField_<Variant_>>()? {
            Some(UnionField_::Type) => {
                let variant = map.next_value()?;
                let key = map.next_key()?;
                match (variant, key) {
                    (Variant_::Generic, Some(Variant_::Generic)) => {
                        let value = map.next_value()?;
                        Diagnostic::Generic(value)
                    }
                    (Variant_::ThreadDump, Some(Variant_::ThreadDump)) => {
                        let value = map.next_value()?;
                        Diagnostic::ThreadDump(value)
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
                    Variant_::Generic => {
                        let value = map.next_value()?;
                        Diagnostic::Generic(value)
                    }
                    Variant_::ThreadDump => {
                        let value = map.next_value()?;
                        Diagnostic::ThreadDump(value)
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
    Generic,
    ThreadDump,
}
impl Variant_ {
    fn as_str(&self) -> &'static str {
        match *self {
            Variant_::Generic => "generic",
            Variant_::ThreadDump => "threadDump",
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
            "generic" => Variant_::Generic,
            "threadDump" => Variant_::ThreadDump,
            value => {
                return Err(de::Error::unknown_variant(value, &["generic", "threadDump"]));
            }
        };
        Ok(v)
    }
}
