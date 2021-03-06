use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StackFrameV1 {
    address: Option<String>,
    procedure: Option<String>,
    file: Option<String>,
    line: Option<i32>,
    params: std::collections::BTreeMap<String, conjure_object::Any>,
}
impl StackFrameV1 {
    /// Returns a new builder.
    #[inline]
    pub fn builder() -> BuilderStage0 {
        Default::default()
    }
    ///The address of the execution point of this stack frame. This is a string because a safelong can't represent the full 64 bit address space.
    #[inline]
    pub fn address(&self) -> Option<&str> {
        self.address.as_ref().map(|o| &**o)
    }
    ///The identifier of the procedure containing the execution point of this stack frame. This is a fully qualified method name in Java and a demangled symbol name in native code, for example. Note that procedure names may include unsafe information if a service is, for exmaple, running user-defined code. It must be safely redacted.
    #[inline]
    pub fn procedure(&self) -> Option<&str> {
        self.procedure.as_ref().map(|o| &**o)
    }
    ///The name of the file containing the source location of the execution point of this stack frame. Note that file names may include unsafe information if a service is, for example, running user-defined code. It must be safely redacted.
    #[inline]
    pub fn file(&self) -> Option<&str> {
        self.file.as_ref().map(|o| &**o)
    }
    ///The line number of the source location of the execution point of this stack frame.
    #[inline]
    pub fn line(&self) -> Option<i32> {
        self.line.as_ref().map(|o| *o)
    }
    ///Other frame-level information.
    #[inline]
    pub fn params(&self) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.params
    }
}
impl Default for BuilderStage0 {
    #[inline]
    fn default() -> Self {
        BuilderStage0 {
            address: Default::default(),
            procedure: Default::default(),
            file: Default::default(),
            line: Default::default(),
            params: Default::default(),
        }
    }
}
impl From<StackFrameV1> for BuilderStage0 {
    #[inline]
    fn from(value: StackFrameV1) -> Self {
        BuilderStage0 {
            address: value.address,
            procedure: value.procedure,
            file: value.file,
            line: value.line,
            params: value.params,
        }
    }
}
///The stage 0 builder for the [`StackFrameV1`] type
#[derive(Debug, Clone)]
pub struct BuilderStage0 {
    address: Option<String>,
    procedure: Option<String>,
    file: Option<String>,
    line: Option<i32>,
    params: std::collections::BTreeMap<String, conjure_object::Any>,
}
impl BuilderStage0 {
    ///The address of the execution point of this stack frame. This is a string because a safelong can't represent the full 64 bit address space.
    #[inline]
    pub fn address<T>(mut self, address: T) -> Self
    where
        T: Into<Option<String>>,
    {
        self.address = address.into();
        self
    }
    ///The identifier of the procedure containing the execution point of this stack frame. This is a fully qualified method name in Java and a demangled symbol name in native code, for example. Note that procedure names may include unsafe information if a service is, for exmaple, running user-defined code. It must be safely redacted.
    #[inline]
    pub fn procedure<T>(mut self, procedure: T) -> Self
    where
        T: Into<Option<String>>,
    {
        self.procedure = procedure.into();
        self
    }
    ///The name of the file containing the source location of the execution point of this stack frame. Note that file names may include unsafe information if a service is, for example, running user-defined code. It must be safely redacted.
    #[inline]
    pub fn file<T>(mut self, file: T) -> Self
    where
        T: Into<Option<String>>,
    {
        self.file = file.into();
        self
    }
    ///The line number of the source location of the execution point of this stack frame.
    #[inline]
    pub fn line<T>(mut self, line: T) -> Self
    where
        T: Into<Option<i32>>,
    {
        self.line = line.into();
        self
    }
    ///Other frame-level information.
    #[inline]
    pub fn params<T>(mut self, params: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.params = params.into_iter().collect();
        self
    }
    ///Other frame-level information.
    #[inline]
    pub fn extend_params<T>(mut self, params: T) -> Self
    where
        T: IntoIterator<Item = (String, conjure_object::Any)>,
    {
        self.params.extend(params);
        self
    }
    ///Other frame-level information.
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
    pub fn build(self) -> StackFrameV1 {
        StackFrameV1 {
            address: self.address,
            procedure: self.procedure,
            file: self.file,
            line: self.line,
            params: self.params,
        }
    }
}
impl ser::Serialize for StackFrameV1 {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut size = 0usize;
        let skip_address = self.address.is_none();
        if !skip_address {
            size += 1;
        }
        let skip_procedure = self.procedure.is_none();
        if !skip_procedure {
            size += 1;
        }
        let skip_file = self.file.is_none();
        if !skip_file {
            size += 1;
        }
        let skip_line = self.line.is_none();
        if !skip_line {
            size += 1;
        }
        let skip_params = self.params.is_empty();
        if !skip_params {
            size += 1;
        }
        let mut s = s.serialize_struct("StackFrameV1", size)?;
        if skip_address {
            s.skip_field("address")?;
        } else {
            s.serialize_field("address", &self.address)?;
        }
        if skip_procedure {
            s.skip_field("procedure")?;
        } else {
            s.serialize_field("procedure", &self.procedure)?;
        }
        if skip_file {
            s.skip_field("file")?;
        } else {
            s.serialize_field("file", &self.file)?;
        }
        if skip_line {
            s.skip_field("line")?;
        } else {
            s.serialize_field("line", &self.line)?;
        }
        if skip_params {
            s.skip_field("params")?;
        } else {
            s.serialize_field("params", &self.params)?;
        }
        s.end()
    }
}
impl<'de> de::Deserialize<'de> for StackFrameV1 {
    fn deserialize<D>(d: D) -> Result<StackFrameV1, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_struct(
            "StackFrameV1",
            &["address", "procedure", "file", "line", "params"],
            Visitor_,
        )
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = StackFrameV1;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("map")
    }
    fn visit_map<A>(self, mut map_: A) -> Result<StackFrameV1, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut address = None;
        let mut procedure = None;
        let mut file = None;
        let mut line = None;
        let mut params = None;
        while let Some(field_) = map_.next_key()? {
            match field_ {
                Field_::Address => address = Some(map_.next_value()?),
                Field_::Procedure => procedure = Some(map_.next_value()?),
                Field_::File => file = Some(map_.next_value()?),
                Field_::Line => line = Some(map_.next_value()?),
                Field_::Params => params = Some(map_.next_value()?),
                Field_::Unknown_ => {
                    map_.next_value::<de::IgnoredAny>()?;
                }
            }
        }
        let address = match address {
            Some(v) => v,
            None => Default::default(),
        };
        let procedure = match procedure {
            Some(v) => v,
            None => Default::default(),
        };
        let file = match file {
            Some(v) => v,
            None => Default::default(),
        };
        let line = match line {
            Some(v) => v,
            None => Default::default(),
        };
        let params = match params {
            Some(v) => v,
            None => Default::default(),
        };
        Ok(StackFrameV1 {
            address,
            procedure,
            file,
            line,
            params,
        })
    }
}
enum Field_ {
    Address,
    Procedure,
    File,
    Line,
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
            "address" => Field_::Address,
            "procedure" => Field_::Procedure,
            "file" => Field_::File,
            "line" => Field_::Line,
            "params" => Field_::Params,
            _ => Field_::Unknown_,
        };
        Ok(v)
    }
}
