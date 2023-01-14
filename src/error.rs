use std::fmt::{self, Display};

#[derive(Debug)]
pub struct MultiError {
    errors: Vec<anyhow::Error>,
}

impl MultiError {
    pub fn into_anyhow(mut self) -> anyhow::Error {
        if self.errors.len() == 1 {
            self.errors.remove(0)
        } else {
            self.into()
        }
    }
}

impl Display for MultiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        assert!(!self.errors.is_empty());

        for (i, e) in self.errors.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            writeln!(f, "{:?}", e)?;
        }

        Ok(())
    }
}

impl std::error::Error for MultiError {}

pub type MultiResult = Result<(), MultiError>;

pub trait MultiResultExt {
    fn combine(self, other: anyhow::Result<()>) -> MultiResult;
}

impl MultiResultExt for Result<(), anyhow::Error> {
    fn combine(self, other: anyhow::Result<()>) -> MultiResult {
        let mut errors = vec![];
        if let Err(e) = self {
            errors.push(e);
        }
        if let Err(e) = other {
            errors.push(e);
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(MultiError { errors })
        }
    }
}

impl MultiResultExt for MultiResult {
    fn combine(self, other: anyhow::Result<()>) -> MultiResult {
        let mut errors = vec![];
        if let Err(mut e) = self {
            errors.append(&mut e.errors);
        }
        if let Err(e) = other {
            errors.push(e);
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(MultiError { errors })
        }
    }
}
