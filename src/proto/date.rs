use chrono::NaiveDateTime;
use serde::{de::Error, Deserializer, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct UnixDate(pub(crate) NaiveDateTime);

impl serde::Serialize for UnixDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.timestamp_millis().to_string())
    }
}

impl<'de> serde::Deserialize<'de> for UnixDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = String::deserialize(deserializer)?;
        let millis = millis
            .parse()
            .map_err(|e| D::Error::custom(format!("invalid time: {e}")))?;
        match NaiveDateTime::from_timestamp_millis(millis) {
            Some(dt) => Ok(Self(dt)),
            None => Err(D::Error::custom(format!("invalid time: {millis}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::proto::testing::{assert_deser_error, assert_roundtrip};

    use super::*;

    #[test]
    fn test_unix_date_roundtrip() {
        assert_roundtrip(UnixDate(
            NaiveDateTime::from_timestamp_millis(1337).unwrap(),
        ));

        assert_deser_error::<UnixDate>(
            &format!(r#""{}""#, i64::MIN),
            "invalid time: -9223372036854775808",
        );
    }
}
