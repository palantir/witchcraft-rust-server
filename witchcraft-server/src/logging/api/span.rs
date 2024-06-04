use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
///A Zipkin-compatible Span object.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[conjure_object::private::staged_builder::staged_builder]
#[builder(crate = conjure_object::private::staged_builder, update, inline)]
pub struct Span {
    #[builder(into)]
    trace_id: String,
    #[builder(into)]
    id: String,
    #[builder(into)]
    name: String,
    #[builder(default, into)]
    parent_id: Option<String>,
    timestamp: conjure_object::SafeLong,
    duration: conjure_object::SafeLong,
    #[builder(default, list(item(type = super::Annotation)))]
    annotations: Vec<super::Annotation>,
    #[builder(default, map(key(type = String, into), value(type = String, into)))]
    tags: std::collections::BTreeMap<String, String>,
}
impl Span {
    ///16-digit hex trace identifier
    #[inline]
    pub fn trace_id(&self) -> &str {
        &*self.trace_id
    }
    ///16-digit hex span identifier
    #[inline]
    pub fn id(&self) -> &str {
        &*self.id
    }
    ///Name of the span (typically the operation/RPC/method name for corresponding to this span)
    #[inline]
    pub fn name(&self) -> &str {
        &*self.name
    }
    ///16-digit hex identifer of the parent span
    #[inline]
    pub fn parent_id(&self) -> Option<&str> {
        self.parent_id.as_ref().map(|o| &**o)
    }
    ///Timestamp of the start of this span (epoch microsecond value)
    #[inline]
    pub fn timestamp(&self) -> conjure_object::SafeLong {
        self.timestamp
    }
    ///Duration of this span (microseconds)
    #[inline]
    pub fn duration(&self) -> conjure_object::SafeLong {
        self.duration
    }
    #[inline]
    pub fn annotations(&self) -> &[super::Annotation] {
        &*self.annotations
    }
    ///Additional dimensions that describe the instance of the trace span
    #[inline]
    pub fn tags(&self) -> &std::collections::BTreeMap<String, String> {
        &self.tags
    }
}
impl ser::Serialize for Span {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut size = 5usize;
        let skip_parent_id = self.parent_id.is_none();
        if !skip_parent_id {
            size += 1;
        }
        let skip_annotations = self.annotations.is_empty();
        if !skip_annotations {
            size += 1;
        }
        let skip_tags = self.tags.is_empty();
        if !skip_tags {
            size += 1;
        }
        let mut s = s.serialize_struct("Span", size)?;
        s.serialize_field("traceId", &self.trace_id)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("name", &self.name)?;
        if skip_parent_id {
            s.skip_field("parentId")?;
        } else {
            s.serialize_field("parentId", &self.parent_id)?;
        }
        s.serialize_field("timestamp", &self.timestamp)?;
        s.serialize_field("duration", &self.duration)?;
        if skip_annotations {
            s.skip_field("annotations")?;
        } else {
            s.serialize_field("annotations", &self.annotations)?;
        }
        if skip_tags {
            s.skip_field("tags")?;
        } else {
            s.serialize_field("tags", &self.tags)?;
        }
        s.end()
    }
}
impl<'de> de::Deserialize<'de> for Span {
    fn deserialize<D>(d: D) -> Result<Span, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_struct(
            "Span",
            &[
                "traceId",
                "id",
                "name",
                "parentId",
                "timestamp",
                "duration",
                "annotations",
                "tags",
            ],
            Visitor_,
        )
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = Span;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("map")
    }
    fn visit_map<A>(self, mut map_: A) -> Result<Span, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut trace_id = None;
        let mut id = None;
        let mut name = None;
        let mut parent_id = None;
        let mut timestamp = None;
        let mut duration = None;
        let mut annotations = None;
        let mut tags = None;
        while let Some(field_) = map_.next_key()? {
            match field_ {
                Field_::TraceId => trace_id = Some(map_.next_value()?),
                Field_::Id => id = Some(map_.next_value()?),
                Field_::Name => name = Some(map_.next_value()?),
                Field_::ParentId => parent_id = Some(map_.next_value()?),
                Field_::Timestamp => timestamp = Some(map_.next_value()?),
                Field_::Duration => duration = Some(map_.next_value()?),
                Field_::Annotations => annotations = Some(map_.next_value()?),
                Field_::Tags => tags = Some(map_.next_value()?),
                Field_::Unknown_ => {
                    map_.next_value::<de::IgnoredAny>()?;
                }
            }
        }
        let trace_id = match trace_id {
            Some(v) => v,
            None => return Err(de::Error::missing_field("traceId")),
        };
        let id = match id {
            Some(v) => v,
            None => return Err(de::Error::missing_field("id")),
        };
        let name = match name {
            Some(v) => v,
            None => return Err(de::Error::missing_field("name")),
        };
        let parent_id = match parent_id {
            Some(v) => v,
            None => Default::default(),
        };
        let timestamp = match timestamp {
            Some(v) => v,
            None => return Err(de::Error::missing_field("timestamp")),
        };
        let duration = match duration {
            Some(v) => v,
            None => return Err(de::Error::missing_field("duration")),
        };
        let annotations = match annotations {
            Some(v) => v,
            None => Default::default(),
        };
        let tags = match tags {
            Some(v) => v,
            None => Default::default(),
        };
        Ok(Span {
            trace_id,
            id,
            name,
            parent_id,
            timestamp,
            duration,
            annotations,
            tags,
        })
    }
}
enum Field_ {
    TraceId,
    Id,
    Name,
    ParentId,
    Timestamp,
    Duration,
    Annotations,
    Tags,
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
            "traceId" => Field_::TraceId,
            "id" => Field_::Id,
            "name" => Field_::Name,
            "parentId" => Field_::ParentId,
            "timestamp" => Field_::Timestamp,
            "duration" => Field_::Duration,
            "annotations" => Field_::Annotations,
            "tags" => Field_::Tags,
            _ => Field_::Unknown_,
        };
        Ok(v)
    }
}
