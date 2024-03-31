#[track_caller]
pub(crate) fn assert_roundtrip<T>(orig: T)
where
    T: Eq + std::fmt::Debug + serde::Serialize + serde::de::DeserializeOwned,
{
    let s = serde_json::to_string(&orig).expect("serialize");
    let recovered = serde_json::from_str(&s).expect("deserialize");
    assert_eq!(orig, recovered);
}

#[track_caller]
pub(crate) fn assert_deser_error<T>(s: &str, expected: &str)
where
    T: std::fmt::Debug + serde::de::DeserializeOwned,
{
    let err = serde_json::from_str::<T>(s).expect_err("no error");
    assert_eq!(err.to_string(), expected);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{de::Error, Deserializer, Serializer};

    #[test]
    fn test_assert_roundtrip_ok() {
        assert_roundtrip(Helper(1));
    }

    #[test]
    #[should_panic(expected = "assertion")]
    fn test_assert_roundtrip_fail() {
        assert_roundtrip(Helper(100));
    }

    #[test]
    fn test_assert_deser_error_ok() {
        assert_deser_error::<Helper>("0", "foo");
    }

    #[test]
    #[should_panic(expected = "no error")]
    fn test_assert_deser_error_fail() {
        assert_deser_error::<Helper>("1", "foo");
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Helper(u8);

    impl serde::Serialize for Helper {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_u8(self.0)
        }
    }

    impl<'de> serde::Deserialize<'de> for Helper {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let i = u8::deserialize(deserializer)?;
            if i == 0 {
                Err(D::Error::custom("foo".to_owned()))
            } else if i < 10 {
                Ok(Self(i))
            } else {
                Ok(Self(0))
            }
        }
    }
}
