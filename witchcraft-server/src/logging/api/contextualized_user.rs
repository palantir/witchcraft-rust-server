use conjure_object::serde::{ser, de};
use conjure_object::serde::ser::SerializeStruct as SerializeStruct_;
use std::fmt;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextualizedUser {
    uid: super::UserId,
    user_name: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    groups: Vec<String>,
    realm: Option<String>,
}
impl ContextualizedUser {
    /// Returns a new builder.
    #[inline]
    pub fn builder() -> BuilderStage0 {
        Default::default()
    }
    #[inline]
    pub fn uid(&self) -> &super::UserId {
        &self.uid
    }
    #[inline]
    pub fn user_name(&self) -> Option<&str> {
        self.user_name.as_ref().map(|o| &**o)
    }
    #[inline]
    pub fn first_name(&self) -> Option<&str> {
        self.first_name.as_ref().map(|o| &**o)
    }
    #[inline]
    pub fn last_name(&self) -> Option<&str> {
        self.last_name.as_ref().map(|o| &**o)
    }
    #[inline]
    pub fn groups(&self) -> &[String] {
        &*self.groups
    }
    #[inline]
    pub fn realm(&self) -> Option<&str> {
        self.realm.as_ref().map(|o| &**o)
    }
}
impl Default for BuilderStage0 {
    #[inline]
    fn default() -> Self {
        BuilderStage0 {}
    }
}
impl From<ContextualizedUser> for BuilderStage1 {
    #[inline]
    fn from(value: ContextualizedUser) -> Self {
        BuilderStage1 {
            uid: value.uid,
            user_name: value.user_name,
            first_name: value.first_name,
            last_name: value.last_name,
            groups: value.groups,
            realm: value.realm,
        }
    }
}
///The stage 0 builder for the [`ContextualizedUser`] type
#[derive(Debug, Clone)]
pub struct BuilderStage0 {}
impl BuilderStage0 {
    #[inline]
    pub fn uid(self, uid: super::UserId) -> BuilderStage1 {
        BuilderStage1 {
            uid: uid,
            user_name: Default::default(),
            first_name: Default::default(),
            last_name: Default::default(),
            groups: Default::default(),
            realm: Default::default(),
        }
    }
}
///The stage 1 builder for the [`ContextualizedUser`] type
#[derive(Debug, Clone)]
pub struct BuilderStage1 {
    uid: super::UserId,
    user_name: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    groups: Vec<String>,
    realm: Option<String>,
}
impl BuilderStage1 {
    #[inline]
    pub fn uid(mut self, uid: super::UserId) -> Self {
        self.uid = uid;
        self
    }
    #[inline]
    pub fn user_name<T>(mut self, user_name: T) -> Self
    where
        T: Into<Option<String>>,
    {
        self.user_name = user_name.into();
        self
    }
    #[inline]
    pub fn first_name<T>(mut self, first_name: T) -> Self
    where
        T: Into<Option<String>>,
    {
        self.first_name = first_name.into();
        self
    }
    #[inline]
    pub fn last_name<T>(mut self, last_name: T) -> Self
    where
        T: Into<Option<String>>,
    {
        self.last_name = last_name.into();
        self
    }
    #[inline]
    pub fn groups<T>(mut self, groups: T) -> Self
    where
        T: IntoIterator<Item = String>,
    {
        self.groups = groups.into_iter().collect();
        self
    }
    #[inline]
    pub fn extend_groups<T>(mut self, groups: T) -> Self
    where
        T: IntoIterator<Item = String>,
    {
        self.groups.extend(groups);
        self
    }
    #[inline]
    pub fn push_groups<T>(mut self, value: T) -> Self
    where
        T: Into<String>,
    {
        self.groups.push(value.into());
        self
    }
    #[inline]
    pub fn realm<T>(mut self, realm: T) -> Self
    where
        T: Into<Option<String>>,
    {
        self.realm = realm.into();
        self
    }
    /// Consumes the builder, constructing a new instance of the type.
    #[inline]
    pub fn build(self) -> ContextualizedUser {
        ContextualizedUser {
            uid: self.uid,
            user_name: self.user_name,
            first_name: self.first_name,
            last_name: self.last_name,
            groups: self.groups,
            realm: self.realm,
        }
    }
}
impl ser::Serialize for ContextualizedUser {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut size = 1usize;
        let skip_user_name = self.user_name.is_none();
        if !skip_user_name {
            size += 1;
        }
        let skip_first_name = self.first_name.is_none();
        if !skip_first_name {
            size += 1;
        }
        let skip_last_name = self.last_name.is_none();
        if !skip_last_name {
            size += 1;
        }
        let skip_groups = self.groups.is_empty();
        if !skip_groups {
            size += 1;
        }
        let skip_realm = self.realm.is_none();
        if !skip_realm {
            size += 1;
        }
        let mut s = s.serialize_struct("ContextualizedUser", size)?;
        s.serialize_field("uid", &self.uid)?;
        if skip_user_name {
            s.skip_field("userName")?;
        } else {
            s.serialize_field("userName", &self.user_name)?;
        }
        if skip_first_name {
            s.skip_field("firstName")?;
        } else {
            s.serialize_field("firstName", &self.first_name)?;
        }
        if skip_last_name {
            s.skip_field("lastName")?;
        } else {
            s.serialize_field("lastName", &self.last_name)?;
        }
        if skip_groups {
            s.skip_field("groups")?;
        } else {
            s.serialize_field("groups", &self.groups)?;
        }
        if skip_realm {
            s.skip_field("realm")?;
        } else {
            s.serialize_field("realm", &self.realm)?;
        }
        s.end()
    }
}
impl<'de> de::Deserialize<'de> for ContextualizedUser {
    fn deserialize<D>(d: D) -> Result<ContextualizedUser, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_struct(
            "ContextualizedUser",
            &["uid", "userName", "firstName", "lastName", "groups", "realm"],
            Visitor_,
        )
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = ContextualizedUser;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("map")
    }
    fn visit_map<A>(self, mut map_: A) -> Result<ContextualizedUser, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut uid = None;
        let mut user_name = None;
        let mut first_name = None;
        let mut last_name = None;
        let mut groups = None;
        let mut realm = None;
        while let Some(field_) = map_.next_key()? {
            match field_ {
                Field_::Uid => uid = Some(map_.next_value()?),
                Field_::UserName => user_name = Some(map_.next_value()?),
                Field_::FirstName => first_name = Some(map_.next_value()?),
                Field_::LastName => last_name = Some(map_.next_value()?),
                Field_::Groups => groups = Some(map_.next_value()?),
                Field_::Realm => realm = Some(map_.next_value()?),
                Field_::Unknown_ => {
                    map_.next_value::<de::IgnoredAny>()?;
                }
            }
        }
        let uid = match uid {
            Some(v) => v,
            None => return Err(de::Error::missing_field("uid")),
        };
        let user_name = match user_name {
            Some(v) => v,
            None => Default::default(),
        };
        let first_name = match first_name {
            Some(v) => v,
            None => Default::default(),
        };
        let last_name = match last_name {
            Some(v) => v,
            None => Default::default(),
        };
        let groups = match groups {
            Some(v) => v,
            None => Default::default(),
        };
        let realm = match realm {
            Some(v) => v,
            None => Default::default(),
        };
        Ok(ContextualizedUser {
            uid,
            user_name,
            first_name,
            last_name,
            groups,
            realm,
        })
    }
}
enum Field_ {
    Uid,
    UserName,
    FirstName,
    LastName,
    Groups,
    Realm,
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
            "uid" => Field_::Uid,
            "userName" => Field_::UserName,
            "firstName" => Field_::FirstName,
            "lastName" => Field_::LastName,
            "groups" => Field_::Groups,
            "realm" => Field_::Realm,
            _ => Field_::Unknown_,
        };
        Ok(v)
    }
}
