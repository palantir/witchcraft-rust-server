use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use conjure_object::serde::{de, ser};
use std::fmt;
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct ThreadDumpV1 {
    threads: Vec<super::ThreadInfoV1>,
}
impl ThreadDumpV1 {
    #[doc = r" Constructs a new instance of the type."]
    #[inline]
    pub fn new<T>(threads: T) -> ThreadDumpV1
    where
        T: IntoIterator<Item = super::ThreadInfoV1>,
    {
        ThreadDumpV1 {
            threads: threads.into_iter().collect(),
        }
    }
    #[doc = r" Returns a new builder."]
    #[inline]
    pub fn builder() -> BuilderStage0 {
        Default::default()
    }
    #[doc = "Information about each of the threads in the thread dump. \"Thread\" may refer to a userland thread such as a goroutine, or an OS-level thread."]
    #[inline]
    pub fn threads(&self) -> &[super::ThreadInfoV1] {
        &*self.threads
    }
}
impl Default for BuilderStage0 {
    #[inline]
    fn default() -> Self {
        BuilderStage0 {
            threads: Default::default(),
        }
    }
}
impl From<ThreadDumpV1> for BuilderStage0 {
    #[inline]
    fn from(value: ThreadDumpV1) -> Self {
        BuilderStage0 {
            threads: value.threads,
        }
    }
}
#[doc = "The stage 0 builder for the [`ThreadDumpV1`] type"]
#[derive(Debug, Clone)]
pub struct BuilderStage0 {
    threads: Vec<super::ThreadInfoV1>,
}
impl BuilderStage0 {
    #[doc = "Information about each of the threads in the thread dump. \"Thread\" may refer to a userland thread such as a goroutine, or an OS-level thread."]
    #[inline]
    pub fn threads<T>(mut self, threads: T) -> Self
    where
        T: IntoIterator<Item = super::ThreadInfoV1>,
    {
        self.threads = threads.into_iter().collect();
        self
    }
    #[doc = "Information about each of the threads in the thread dump. \"Thread\" may refer to a userland thread such as a goroutine, or an OS-level thread."]
    #[inline]
    pub fn extend_threads<T>(mut self, threads: T) -> Self
    where
        T: IntoIterator<Item = super::ThreadInfoV1>,
    {
        self.threads.extend(threads);
        self
    }
    #[doc = "Information about each of the threads in the thread dump. \"Thread\" may refer to a userland thread such as a goroutine, or an OS-level thread."]
    #[inline]
    pub fn push_threads(mut self, value: super::ThreadInfoV1) -> Self {
        self.threads.push(value);
        self
    }
    #[doc = r" Consumes the builder, constructing a new instance of the type."]
    #[inline]
    pub fn build(self) -> ThreadDumpV1 {
        ThreadDumpV1 {
            threads: self.threads,
        }
    }
}
impl ser::Serialize for ThreadDumpV1 {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut size = 0usize;
        let skip_threads = self.threads.is_empty();
        if !skip_threads {
            size += 1;
        }
        let mut s = s.serialize_struct("ThreadDumpV1", size)?;
        if skip_threads {
            s.skip_field("threads")?;
        } else {
            s.serialize_field("threads", &self.threads)?;
        }
        s.end()
    }
}
impl<'de> de::Deserialize<'de> for ThreadDumpV1 {
    fn deserialize<D>(d: D) -> Result<ThreadDumpV1, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_struct("ThreadDumpV1", &["threads"], Visitor_)
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = ThreadDumpV1;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("map")
    }
    fn visit_map<A>(self, mut map_: A) -> Result<ThreadDumpV1, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut threads = None;
        while let Some(field_) = map_.next_key()? {
            match field_ {
                Field_::Threads => threads = Some(map_.next_value()?),
                Field_::Unknown_ => {
                    map_.next_value::<de::IgnoredAny>()?;
                }
            }
        }
        let threads = match threads {
            Some(v) => v,
            None => Default::default(),
        };
        Ok(ThreadDumpV1 { threads })
    }
}
enum Field_ {
    Threads,
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
            "threads" => Field_::Threads,
            _ => Field_::Unknown_,
        };
        Ok(v)
    }
}
