use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Format<const F: u8>;

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
        use serde::de::Error;

        let s = String::deserialize(deserializer)?;
        let f: u8 = s.parse().map_err(|e| D::Error::custom(e))?;
        if f == F {
            Ok(Self)
        } else {
            Err(D::Error::custom(format!("invalid format: {f}")))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KdfVersion {
    Bcrypt,
    Argon2id,
}

impl serde::Serialize for KdfVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            Self::Bcrypt => "0",
            Self::Argon2id => "1",
        };
        serializer.serialize_str(s)
    }
}

impl<'de> serde::Deserialize<'de> for KdfVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "0" => Ok(Self::Bcrypt),
            "1" => Ok(Self::Argon2id),
            s => Err(D::Error::custom(format!("invalid KDF version: {s}"))),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaltServiceRequest {
    #[serde(rename = "_format")]
    pub format: Format<0>,

    pub mail_address: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaltServiceResponse {
    #[serde(rename = "_format")]
    pub format: Format<0>,

    pub kdf_version: KdfVersion,

    pub salt: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_format() {
        assert_roundtrip(Format::<0>);
        assert_roundtrip(Format::<1>);
        assert_roundtrip(Format::<2>);
        assert_roundtrip(Format::<255>);

        assert_deser_error::<Format<1>>(r#""0""#, "invalid format: 0");
    }

    #[test]
    fn test_roundtrip_kdf_version() {
        assert_roundtrip(KdfVersion::Bcrypt);
        assert_roundtrip(KdfVersion::Argon2id);
    }

    #[track_caller]
    fn assert_roundtrip<T>(orig: T)
    where
        T: Eq + std::fmt::Debug + serde::Serialize + serde::de::DeserializeOwned,
    {
        let s = serde_json::to_string(&orig).expect("serialize");
        let recovered = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(orig, recovered);
    }

    #[track_caller]
    fn assert_deser_error<T>(s: &str, expected: &str)
    where
        T: std::fmt::Debug + serde::de::DeserializeOwned,
    {
        let err = serde_json::from_str::<T>(s).expect_err("deserialize error");
        assert_eq!(err.to_string(), expected);
    }
}
