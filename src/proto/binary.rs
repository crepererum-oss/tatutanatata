use anyhow::{bail, Result};
use base64::prelude::*;
use serde::{de::Error, Deserializer, Serializer};
use std::ops::Deref;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct Base64String(Box<[u8]>);

impl Base64String {
    pub(crate) fn try_new(s: &str) -> Result<Self> {
        let data = BASE64_STANDARD.decode(s)?;
        Ok(Self(data.into()))
    }

    pub(crate) fn base64(&self) -> String {
        BASE64_STANDARD.encode(self.0.as_ref())
    }
}

impl std::fmt::Debug for Base64String {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.base64())
    }
}

impl std::fmt::Display for Base64String {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.base64())
    }
}

impl From<Vec<u8>> for Base64String {
    fn from(value: Vec<u8>) -> Self {
        Self(value.into())
    }
}

impl From<&[u8]> for Base64String {
    fn from(value: &[u8]) -> Self {
        Self(value.into())
    }
}

impl<const N: usize> From<[u8; N]> for Base64String {
    fn from(value: [u8; N]) -> Self {
        Self(value.into())
    }
}

impl<const N: usize> From<&[u8; N]> for Base64String {
    fn from(value: &[u8; N]) -> Self {
        Self(value.to_owned().into())
    }
}

impl AsRef<[u8]> for Base64String {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Deref for Base64String {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl serde::Serialize for Base64String {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.base64())
    }
}

impl<'de> serde::Deserialize<'de> for Base64String {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::try_new(&s).map_err(D::Error::custom)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct Base64Url(Base64String);

impl Base64Url {
    pub(crate) fn try_new(s: &str) -> Result<Self> {
        let mut s = s.replace('-', "+").replace('_', "/");
        match s.len() % 4 {
            0 => {}
            2 => {
                s.push_str("==");
            }
            3 => {
                s.push('=');
            }
            _ => {
                bail!("invalid base64 URL")
            }
        }
        Ok(Self(Base64String::try_new(&s)?))
    }

    pub(crate) fn url(&self) -> String {
        self.0
            .base64()
            .replace('+', "-")
            .replace('/', "_")
            .replace('=', "")
    }
}

impl std::fmt::Debug for Base64Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url())
    }
}

impl std::fmt::Display for Base64Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url())
    }
}

impl From<Vec<u8>> for Base64Url {
    fn from(value: Vec<u8>) -> Self {
        Self(value.into())
    }
}

impl From<&[u8]> for Base64Url {
    fn from(value: &[u8]) -> Self {
        Self(value.into())
    }
}

impl<const N: usize> From<[u8; N]> for Base64Url {
    fn from(value: [u8; N]) -> Self {
        Self(value.into())
    }
}

impl<const N: usize> From<&[u8; N]> for Base64Url {
    fn from(value: &[u8; N]) -> Self {
        Self(value.to_owned().into())
    }
}

impl AsRef<[u8]> for Base64Url {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl serde::Serialize for Base64Url {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.url())
    }
}

impl<'de> serde::Deserialize<'de> for Base64Url {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::try_new(&s).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::testing::assert_roundtrip;

    #[test]
    fn test_roundtrip_base64string() {
        assert_roundtrip(Base64String::from(b""), r#""""#);
        assert_roundtrip(Base64String::from(b"foo"), r#""Zm9v""#);
    }

    #[test]
    fn test_roundtrip_base64url() {
        assert_roundtrip(Base64Url::from(b""), r#""""#);
        assert_roundtrip(Base64Url::from(b"foo"), r#""Zm9v""#);
    }
}
