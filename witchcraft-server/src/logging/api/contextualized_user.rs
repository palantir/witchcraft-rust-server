#[derive(
    Debug,
    Clone,
    conjure_object::serde::Serialize,
    conjure_object::serde::Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash
)]
#[serde(crate = "conjure_object::serde")]
#[conjure_object::private::staged_builder::staged_builder]
#[builder(crate = conjure_object::private::staged_builder, update, inline)]
pub struct ContextualizedUser {
    #[serde(rename = "uid")]
    uid: super::UserId,
    #[builder(default, into)]
    #[serde(rename = "userName", skip_serializing_if = "Option::is_none", default)]
    user_name: Option<String>,
    #[builder(default, into)]
    #[serde(rename = "firstName", skip_serializing_if = "Option::is_none", default)]
    first_name: Option<String>,
    #[builder(default, into)]
    #[serde(rename = "lastName", skip_serializing_if = "Option::is_none", default)]
    last_name: Option<String>,
    #[builder(default, list(item(type = String, into)))]
    #[serde(rename = "groups", skip_serializing_if = "Vec::is_empty", default)]
    groups: Vec<String>,
    #[builder(default, into)]
    #[serde(rename = "realm", skip_serializing_if = "Option::is_none", default)]
    realm: Option<String>,
}
impl ContextualizedUser {
    /// Constructs a new instance of the type.
    #[inline]
    pub fn new(uid: super::UserId) -> Self {
        Self::builder().uid(uid).build()
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
