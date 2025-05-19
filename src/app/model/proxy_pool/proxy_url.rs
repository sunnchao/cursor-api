use reqwest::Proxy;
use rkyv::{Archive, Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// 一个可以被Archive的字符串化URL
#[derive(Clone, Archive, Deserialize, Serialize)]
#[archive(compare(PartialEq))]
#[repr(transparent)]
pub struct StringUrl(String);

impl StringUrl {
    pub fn into_proxy(self) -> Result<Proxy, reqwest::Error> {
        Proxy::all(&self.0)
    }

    pub fn as_proxy(&self) -> Result<Proxy, reqwest::Error> {
        Proxy::all(&self.0)
    }
}

impl TryFrom<StringUrl> for Proxy {
    type Error = reqwest::Error;

    fn try_from(string_url: StringUrl) -> Result<Self, Self::Error> {
        string_url.into_proxy()
    }
}

impl fmt::Display for StringUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for StringUrl {
    type Err = reqwest::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Proxy::all(s)?;
        Ok(Self(s.to_string()))
    }
}

impl PartialEq for StringUrl {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for StringUrl {}

impl std::hash::Hash for StringUrl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
