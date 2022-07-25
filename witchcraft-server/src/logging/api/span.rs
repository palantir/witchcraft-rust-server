use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
///A Zipkin-compatible Span object.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    trace_id: String,
    id: String,
    name: String,
    parent_id: Option<String>,
    timestamp: conjure_object::SafeLong,
    duration: conjure_object::SafeLong,
    annotations: Vec<super::Annotation>,
    tags: std::collections::BTreeMap<String, String>,
}
impl Span {
    /// Returns a new builder.
    #[inline]
    pub fn builder() -> BuilderStage0 {
        Default::default()
    }
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
impl Default for BuilderStage0 {
    #[inline]
    fn default() -> Self {
        BuilderStage0 {}
    }
}
impl From<Span> for BuilderStage5 {
    #[inline]
    fn from(value: Span) -> Self {
        BuilderStage5 {
            trace_id: value.trace_id,
            id: value.id,
            name: value.name,
            parent_id: value.parent_id,
            timestamp: value.timestamp,
            duration: value.duration,
            annotations: value.annotations,
            tags: value.tags,
        }
    }
}
///The stage 0 builder for the [`Span`] type
#[derive(Debug, Clone)]
pub struct BuilderStage0 {}
impl BuilderStage0 {
    ///16-digit hex trace identifier
    #[inline]
    pub fn trace_id<T>(self, trace_id: T) -> BuilderStage1
    where
        T: Into<String>,
    {
        BuilderStage1 {
            trace_id: trace_id.into(),
        }
    }
}
///The stage 1 builder for the [`Span`] type
#[derive(Debug, Clone)]
pub struct BuilderStage1 {
    trace_id: String,
}
impl BuilderStage1 {
    ///16-digit hex span identifier
    #[inline]
    pub fn id<T>(self, id: T) -> BuilderStage2
    where
        T: Into<String>,
    {
        BuilderStage2 {
            trace_id: self.trace_id,
            id: id.into(),
        }
    }
}
///The stage 2 builder for the [`Span`] type
#[derive(Debug, Clone)]
pub struct BuilderStage2 {
    trace_id: String,
    id: String,
}
impl BuilderStage2 {
    ///Name of the span (typically the operation/RPC/method name for corresponding to this span)
    #[inline]
    pub fn name<T>(self, name: T) -> BuilderStage3
    where
        T: Into<String>,
    {
        BuilderStage3 {
            trace_id: self.trace_id,
            id: self.id,
            name: name.into(),
        }
    }
}
///The stage 3 builder for the [`Span`] type
#[derive(Debug, Clone)]
pub struct BuilderStage3 {
    trace_id: String,
    id: String,
    name: String,
}
impl BuilderStage3 {
    ///Timestamp of the start of this span (epoch microsecond value)
    #[inline]
    pub fn timestamp(self, timestamp: conjure_object::SafeLong) -> BuilderStage4 {
        BuilderStage4 {
            trace_id: self.trace_id,
            id: self.id,
            name: self.name,
            timestamp: timestamp,
        }
    }
}
///The stage 4 builder for the [`Span`] type
#[derive(Debug, Clone)]
pub struct BuilderStage4 {
    trace_id: String,
    id: String,
    name: String,
    timestamp: conjure_object::SafeLong,
}
impl BuilderStage4 {
    ///Duration of this span (microseconds)
    #[inline]
    pub fn duration(self, duration: conjure_object::SafeLong) -> BuilderStage5 {
        BuilderStage5 {
            trace_id: self.trace_id,
            id: self.id,
            name: self.name,
            timestamp: self.timestamp,
            duration: duration,
            parent_id: Default::default(),
            annotations: Default::default(),
            tags: Default::default(),
        }
    }
}
///The stage 5 builder for the [`Span`] type
#[derive(Debug, Clone)]
pub struct BuilderStage5 {
    trace_id: String,
    id: String,
    name: String,
    timestamp: conjure_object::SafeLong,
    duration: conjure_object::SafeLong,
    parent_id: Option<String>,
    annotations: Vec<super::Annotation>,
    tags: std::collections::BTreeMap<String, String>,
}
impl BuilderStage5 {
    ///16-digit hex trace identifier
    #[inline]
    pub fn trace_id<T>(mut self, trace_id: T) -> Self
    where
        T: Into<String>,
    {
        self.trace_id = trace_id.into();
        self
    }
    ///16-digit hex span identifier
    #[inline]
    pub fn id<T>(mut self, id: T) -> Self
    where
        T: Into<String>,
    {
        self.id = id.into();
        self
    }
    ///Name of the span (typically the operation/RPC/method name for corresponding to this span)
    #[inline]
    pub fn name<T>(mut self, name: T) -> Self
    where
        T: Into<String>,
    {
        self.name = name.into();
        self
    }
    ///Timestamp of the start of this span (epoch microsecond value)
    #[inline]
    pub fn timestamp(mut self, timestamp: conjure_object::SafeLong) -> Self {
        self.timestamp = timestamp;
        self
    }
    ///Duration of this span (microseconds)
    #[inline]
    pub fn duration(mut self, duration: conjure_object::SafeLong) -> Self {
        self.duration = duration;
        self
    }
    ///16-digit hex identifer of the parent span
    #[inline]
    pub fn parent_id<T>(mut self, parent_id: T) -> Self
    where
        T: Into<Option<String>>,
    {
        self.parent_id = parent_id.into();
        self
    }
    #[inline]
    pub fn annotations<T>(mut self, annotations: T) -> Self
    where
        T: IntoIterator<Item = super::Annotation>,
    {
        self.annotations = annotations.into_iter().collect();
        self
    }
    #[inline]
    pub fn extend_annotations<T>(mut self, annotations: T) -> Self
    where
        T: IntoIterator<Item = super::Annotation>,
    {
        self.annotations.extend(annotations);
        self
    }
    #[inline]
    pub fn push_annotations(mut self, value: super::Annotation) -> Self {
        self.annotations.push(value);
        self
    }
    ///Additional dimensions that describe the instance of the trace span
    #[inline]
    pub fn tags<T>(mut self, tags: T) -> Self
    where
        T: IntoIterator<Item = (String, String)>,
    {
        self.tags = tags.into_iter().collect();
        self
    }
    ///Additional dimensions that describe the instance of the trace span
    #[inline]
    pub fn extend_tags<T>(mut self, tags: T) -> Self
    where
        T: IntoIterator<Item = (String, String)>,
    {
        self.tags.extend(tags);
        self
    }
    ///Additional dimensions that describe the instance of the trace span
    #[inline]
    pub fn insert_tags<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.tags.insert(key.into(), value.into());
        self
    }
    /// Consumes the builder, constructing a new instance of the type.
    #[inline]
    pub fn build(self) -> Span {
        Span {
            trace_id: self.trace_id,
            id: self.id,
            name: self.name,
            parent_id: self.parent_id,
            timestamp: self.timestamp,
            duration: self.duration,
            annotations: self.annotations,
            tags: self.tags,
        }
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
