use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[conjure_object::private::staged_builder::staged_builder]
#[builder(crate = conjure_object::private::staged_builder, update, inline)]
pub struct ThreadDumpV1 {
    #[builder(default, list(item(type = super::ThreadInfoV1)))]
    threads: Vec<super::ThreadInfoV1>,
}
impl ThreadDumpV1 {
    /// Constructs a new instance of the type.
    #[inline]
    pub fn new() -> Self {
        Self::builder().build()
    }
    ///Information about each of the threads in the thread dump. "Thread" may refer to a userland thread such as a goroutine, or an OS-level thread.
    #[inline]
    pub fn threads(&self) -> &[super::ThreadInfoV1] {
        &*self.threads
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
