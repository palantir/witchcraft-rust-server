#[derive(
    Debug,
    Clone,
    conjure_object::serde::Deserialize,
    conjure_object::serde::Serialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default
)]
#[serde(crate = "conjure_object::serde", transparent)]
pub struct UserId(pub String);
impl std::fmt::Display for UserId {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, fmt)
    }
}
impl conjure_object::Plain for UserId {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        conjure_object::Plain::fmt(&self.0, fmt)
    }
}
impl conjure_object::FromPlain for UserId {
    type Err = <String as conjure_object::FromPlain>::Err;
    #[inline]
    fn from_plain(s: &str) -> Result<UserId, Self::Err> {
        conjure_object::FromPlain::from_plain(s).map(UserId)
    }
}
impl std::convert::From<String> for UserId {
    #[inline]
    fn from(v: String) -> Self {
        UserId(std::convert::From::from(v))
    }
}
impl std::ops::Deref for UserId {
    type Target = String;
    #[inline]
    fn deref(&self) -> &String {
        &self.0
    }
}
impl std::ops::DerefMut for UserId {
    #[inline]
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}
