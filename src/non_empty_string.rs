use std::{ops::Deref, str::FromStr};

/// Non-empty [`String`].
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct NonEmptyString(String);

impl std::fmt::Debug for NonEmptyString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

impl std::fmt::Display for NonEmptyString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

impl Deref for NonEmptyString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl AsRef<str> for NonEmptyString {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl FromStr for NonEmptyString {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if s.is_empty() {
            Err("cannot be empty".to_owned())
        } else {
            Ok(Self(s.to_owned()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let non_empty = NonEmptyString::from_str("foo").unwrap();
        assert_eq!(non_empty.as_ref(), "foo");
        assert_eq!(non_empty.deref(), "foo");
        assert_eq!(format!("{}", non_empty), "foo");
        assert_eq!(format!("{:?}", non_empty), r#""foo""#);
    }

    #[test]
    fn test_failure() {
        let err = NonEmptyString::from_str("").unwrap_err();
        assert_eq!(err.to_string(), "cannot be empty");
    }
}
