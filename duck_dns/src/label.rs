use std::{error::Error as StdError, fmt, str::FromStr};

#[derive(Debug)]
pub enum ParseLabelError {
    Empty,
    StartsWithHyphen,
    EndsWithHyphen,
    TooLong,
    BadChar(char),
    IdnaError(idna::Errors),
}

impl fmt::Display for ParseLabelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseLabelError::Empty => write!(f, "domain label can not be empty"),
            ParseLabelError::StartsWithHyphen => {
                write!(f, "domain label can not start with hyphen")
            }
            ParseLabelError::EndsWithHyphen => write!(f, "domain label can not end with hyphen"),
            ParseLabelError::TooLong => write!(f, "domain label too long"),
            ParseLabelError::BadChar(c) => write!(f, "'{}' not allowed in domain label", c),
            ParseLabelError::IdnaError(_) => write!(f, "invalid international domain label"),
        }
    }
}

impl From<idna::Errors> for ParseLabelError {
    fn from(e: idna::Errors) -> Self {
        ParseLabelError::IdnaError(e)
    }
}

impl StdError for ParseLabelError {}

#[derive(Debug, Eq, PartialEq)]
pub struct Label(pub(crate) String);

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for Label {
    type Err = ParseLabelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseLabelError::Empty);
        }
        if s.starts_with('-') {
            return Err(ParseLabelError::StartsWithHyphen);
        }
        if s.ends_with('-') {
            return Err(ParseLabelError::EndsWithHyphen);
        }
        if s.len() > 63 {
            return Err(ParseLabelError::TooLong);
        }
        for c in s.chars() {
            if c.is_ascii() && !(c.is_ascii_alphanumeric() || c == '-') {
                return Err(ParseLabelError::BadChar(c));
            }
        }
        if s.is_ascii() {
            return Ok(Label(s.to_owned()));
        }
        Ok(Label(idna::domain_to_ascii(s)?))
    }
}
