use std::{ops::Deref, str::FromStr};

/// Non-empty [`String`].
#[derive(Clone)]
pub struct NonEmptyString(String);

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
