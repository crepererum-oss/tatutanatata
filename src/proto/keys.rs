use serde::{de::Error, Deserializer, Serializer};
use std::ops::Deref;

use super::binary::Base64String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Key {
    Aes128([u8; 16]),
    Aes256([u8; 32]),
}

impl AsRef<[u8]> for Key {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Aes128(k) => k,
            Self::Aes256(k) => k,
        }
    }
}

impl Deref for Key {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Aes128(k) => k,
            Self::Aes256(k) => k,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EncryptedKey(pub(crate) Key);

impl Deref for EncryptedKey {
    type Target = Key;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl serde::Serialize for EncryptedKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Base64String::from(self.deref().deref()).serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for EncryptedKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let k = OptionalEncryptedKey::deserialize(deserializer)?;

        match k.0 {
            None => Err(D::Error::custom("key must not be empty")),
            Some(k) => Ok(k),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OptionalEncryptedKey(pub(crate) Option<EncryptedKey>);

impl serde::Serialize for OptionalEncryptedKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0 {
            None => Base64String::from(&[]).serialize(serializer),
            Some(k) => Base64String::from(k.0.deref()).serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for OptionalEncryptedKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = Base64String::deserialize(deserializer)?;

        if s.deref().is_empty() {
            Ok(Self(None))
        } else if let Ok(k) = TryInto::<[u8; 16]>::try_into(s.deref()) {
            Ok(Self(Some(EncryptedKey(Key::Aes128(k)))))
        } else if let Ok(k) = TryInto::<[u8; 32]>::try_into(s.deref()) {
            Ok(Self(Some(EncryptedKey(Key::Aes256(k)))))
        } else {
            Err(D::Error::custom(format!(
                "invalid key length: {}",
                s.deref().len()
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::proto::testing::{assert_deser_error, assert_roundtrip};

    use super::*;

    #[test]
    fn test_roundtrip_encrypted_key() {
        assert_roundtrip(EncryptedKey(Key::Aes128([42; 16])));
        assert_roundtrip(EncryptedKey(Key::Aes256([42; 32])));

        assert_deser_error::<EncryptedKey>(r#""""#, "key must not be empty");
        assert_deser_error::<EncryptedKey>(r#""eAo=""#, "invalid key length: 2");
    }

    #[test]
    fn test_roundtrip_optional_encrypted_key() {
        assert_roundtrip(OptionalEncryptedKey(Some(EncryptedKey(Key::Aes128(
            [42; 16],
        )))));
        assert_roundtrip(OptionalEncryptedKey(Some(EncryptedKey(Key::Aes256(
            [42; 32],
        )))));
        assert_roundtrip(OptionalEncryptedKey(None));

        assert_deser_error::<EncryptedKey>(r#""eAo=""#, "invalid key length: 2");
    }
}
