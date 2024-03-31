use anyhow::Result;
use serde::{de::Error, Deserializer, Serializer};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Format<const F: u8>;

impl<const F: u8> serde::Serialize for Format<F> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&F.to_string())
    }
}

impl<'de, const F: u8> serde::Deserialize<'de> for Format<F> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let f: u8 = s.parse().map_err(D::Error::custom)?;
        if f == F {
            Ok(Self)
        } else {
            Err(D::Error::custom(format!("invalid format: {f}")))
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Null;

impl serde::Serialize for Null {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_none()
    }
}

#[cfg(test)]
mod tests {
    use crate::proto::testing::{assert_deser_error, assert_roundtrip};

    use super::*;

    #[test]
    fn test_roundtrip_format() {
        assert_roundtrip(Format::<0>);
        assert_roundtrip(Format::<1>);
        assert_roundtrip(Format::<2>);
        assert_roundtrip(Format::<255>);

        assert_deser_error::<Format<1>>(r#""0""#, "invalid format: 0");
    }
}
