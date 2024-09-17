use anyhow::Result;
use serde::{de::Error, Deserializer, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Number(pub(crate) u64);

impl serde::Serialize for Number {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for Number {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let n = s
            .parse()
            .map_err(|e| D::Error::custom(format!("invalid number: {e}")))?;
        Ok(Self(n))
    }
}

#[cfg(test)]
mod tests {
    use crate::proto::testing::{assert_deser_error, assert_roundtrip};

    use super::*;

    #[test]
    fn test_number_roundtrip() {
        assert_roundtrip(Number(1337), r#""1337""#);

        assert_deser_error::<Number>(
            &format!(r#""{}""#, -1),
            "invalid number: invalid digit found in string",
        );
    }
}
