use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ThreadInfoV1 {
    id: Option<conjure_object::SafeLong>,
    name: Option<String>,
    stack_trace: Vec<super::StackFrameV1>,
    params: std::collections::BTreeMap<String, conjure_object::Any>,
}
impl ThreadInfoV1 {
    /// Returns a new builder.
    #[inline]
    pub fn builder() -> BuilderStage0 {
        Default::default()
    }
    ///The ID of the thread.
    #[inline]
    pub fn id(&self) -> Option<conjure_object::SafeLong> {
        self.id.as_ref().map(|o| *o)
    }
    ///The name of the thread. Note that thread names may include unsafe information such as the path of the HTTP request being processed. It must be safely redacted.
    #[inline]
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|o| &**o)
    }
    ///A list of stack frames for the thread, ordered with the current frame first.
    #[inline]
    pub fn stack_trace(&self) -> &[super::StackFrameV1] {
        &*self.stack_trace
    }
    ///Other thread-level information.
    #[inline]
    pub fn params(&self) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.params
    }
}
impl Default for BuilderStage0 {
    #[inline]
    fn default() -> Self {
        BuilderStage0 {
            id: Default::default(),
            name: Default::default(),
            stack_trace: Default::default(),
            params: Default::default(),
        }
    }
}
impl From<ThreadInfoV1> for BuilderStage0 {
    #[inline]
    fn from(value: ThreadInfoV1) -> Self {
        BuilderStage0 {
            id: value.id,
            name: value.name,
            stack_trace: value.stack_trace,
            params: value.params,
        }
    }
}
///The stage 0 builder for the [`ThreadInfoV1`] type
#[derive(Debug, Clone)]
pub struct BuilderStage0 {
    id: Option<conjure_object::SafeLong>,
    name: Option<String>,
    stack_trace: Vec<super::StackFrameV1>,
    params: std::collections::BTreeMap<String, conjure_object::Any>,
}
impl BuilderStage0 {
    ///The ID of the thread.
    #[inline]
    pub fn id<T>(mut self, id: T) -> Self
    where
        T: Into<Option<conjure_object::SafeLong>>,
    {
        self.id = id.into();
        self
    }
    ///The name of the thread. Note that thread names may include unsafe information such as the path of the HTTP request being processed. It must be safely redacted.
    #[inline]
    pub fn name<T>(mut self, name: T) -> Self
    where
        T: Into<Option<String>>,
    {
        self.name = name.into();
        self
    }
    ///A list of stack frames for the thread, ordered with the current frame first.
    #[inline]
    pub fn stack_trace<T>(mut self, stack_trace: T) -> Self
    where
        T: IntoIterator<Item = super::StackFrameV1>,
    {
        self.stack_trace = stack_trace.into_iter().collect();
        self
    }
    ///A list of stack frames for the thread, ordered with the current frame first.
    #[inline]
    pub fn extend_stack_trace<T>(mut self, stack_trace: T) -> Self
    where
        T: IntoIterator<Item = super::StackFrameV1>,
    {
        self.stack_trace.extend(stack_trace);
        self
    }
    ///A list of stack frames for the thread, ordered with the current frame first.
    #[inline]
    pub fn push_stack_trace(mut self, value: super::StackFrameV1) -> Self {
        self.stack_trace.push(value);
        self
    }
    ///Other thread-level information.
    #[inline]
    pub fn params<T>(mut self, params: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.params = params.into_iter().collect();
        self
    }
    ///Other thread-level information.
    #[inline]
    pub fn extend_params<T>(mut self, params: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.params.extend(params);
        self
    }
    ///Other thread-level information.
    #[inline]
    pub fn insert_params<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: conjure_object::serde::Serialize,
    {
        self.params
            .insert(
                key.into(),
                conjure_object::Any::new(value).expect("value failed to serialize"),
            );
        self
    }
    /// Consumes the builder, constructing a new instance of the type.
    #[inline]
    pub fn build(self) -> ThreadInfoV1 {
        ThreadInfoV1 {
            id: self.id,
            name: self.name,
            stack_trace: self.stack_trace,
            params: self.params,
        }
    }
}
impl ser::Serialize for ThreadInfoV1 {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut size = 0usize;
        let skip_id = self.id.is_none();
        if !skip_id {
            size += 1;
        }
        let skip_name = self.name.is_none();
        if !skip_name {
            size += 1;
        }
        let skip_stack_trace = self.stack_trace.is_empty();
        if !skip_stack_trace {
            size += 1;
        }
        let skip_params = self.params.is_empty();
        if !skip_params {
            size += 1;
        }
        let mut s = s.serialize_struct("ThreadInfoV1", size)?;
        if skip_id {
            s.skip_field("id")?;
        } else {
            s.serialize_field("id", &self.id)?;
        }
        if skip_name {
            s.skip_field("name")?;
        } else {
            s.serialize_field("name", &self.name)?;
        }
        if skip_stack_trace {
            s.skip_field("stackTrace")?;
        } else {
            s.serialize_field("stackTrace", &self.stack_trace)?;
        }
        if skip_params {
            s.skip_field("params")?;
        } else {
            s.serialize_field("params", &self.params)?;
        }
        s.end()
    }
}
impl<'de> de::Deserialize<'de> for ThreadInfoV1 {
    fn deserialize<D>(d: D) -> Result<ThreadInfoV1, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_struct(
            "ThreadInfoV1",
            &["id", "name", "stackTrace", "params"],
            Visitor_,
        )
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = ThreadInfoV1;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("map")
    }
    fn visit_map<A>(self, mut map_: A) -> Result<ThreadInfoV1, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut id = None;
        let mut name = None;
        let mut stack_trace = None;
        let mut params = None;
        while let Some(field_) = map_.next_key()? {
            match field_ {
                Field_::Id => id = Some(map_.next_value()?),
                Field_::Name => name = Some(map_.next_value()?),
                Field_::StackTrace => stack_trace = Some(map_.next_value()?),
                Field_::Params => params = Some(map_.next_value()?),
                Field_::Unknown_ => {
                    map_.next_value::<de::IgnoredAny>()?;
                }
            }
        }
        let id = match id {
            Some(v) => v,
            None => Default::default(),
        };
        let name = match name {
            Some(v) => v,
            None => Default::default(),
        };
        let stack_trace = match stack_trace {
            Some(v) => v,
            None => Default::default(),
        };
        let params = match params {
            Some(v) => v,
            None => Default::default(),
        };
        Ok(ThreadInfoV1 {
            id,
            name,
            stack_trace,
            params,
        })
    }
}
enum Field_ {
    Id,
    Name,
    StackTrace,
    Params,
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
            "id" => Field_::Id,
            "name" => Field_::Name,
            "stackTrace" => Field_::StackTrace,
            "params" => Field_::Params,
            _ => Field_::Unknown_,
        };
        Ok(v)
    }
}
