use conjure_object::serde::{de, ser};
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TokenId(pub String);
impl std::fmt::Display for TokenId {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, fmt)
    }
}
impl conjure_object::Plain for TokenId {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        conjure_object::Plain::fmt(&self.0, fmt)
    }
}
impl conjure_object::FromPlain for TokenId {
    type Err = <String as conjure_object::FromPlain>::Err;
    #[inline]
    fn from_plain(s: &str) -> Result<TokenId, Self::Err> {
        conjure_object::FromPlain::from_plain(s).map(TokenId)
    }
}
impl std::ops::Deref for TokenId {
    type Target = String;
    #[inline]
    fn deref(&self) -> &String {
        &self.0
    }
}
impl std::ops::DerefMut for TokenId {
    #[inline]
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}
impl ser::Serialize for TokenId {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.0.serialize(s)
    }
}
impl<'de> de::Deserialize<'de> for TokenId {
    fn deserialize<D>(d: D) -> Result<TokenId, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        de::Deserialize::deserialize(d).map(TokenId)
    }
}
