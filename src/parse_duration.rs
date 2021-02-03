use std::{convert::TryFrom, error::Error as StdError, fmt, num::ParseIntError, time::Duration};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::one_of,
    combinator::{all_consuming, recognize},
    multi::many0,
    sequence::pair,
};

struct Overflow();

#[derive(Debug)]
pub enum ParseError<'a> {
    UnrecognizedInput(&'a str),
    BadInt(ParseIntError),
    Overflow,
}

impl fmt::Display for ParseError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnrecognizedInput(input) => write!(f, "unrecognized input {:?}", input),
            ParseError::BadInt(e) => e.fmt(f),
            ParseError::Overflow => write!(f, "value out of range"),
        }
    }
}

impl StdError for ParseError<'_> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            ParseError::UnrecognizedInput(_) => None,
            ParseError::BadInt(e) => Some(e),
            ParseError::Overflow => None,
        }
    }
}

impl<'a> From<nom::Err<nom::error::Error<&'a str>>> for ParseError<'a> {
    fn from(e: nom::Err<nom::error::Error<&'a str>>) -> Self {
        match e {
            nom::Err::Error(e) => Self::UnrecognizedInput(e.input),
            _ => panic!("unexpected error variant"),
        }
    }
}

impl From<ParseIntError> for ParseError<'_> {
    fn from(e: ParseIntError) -> Self {
        Self::BadInt(e)
    }
}

impl From<Overflow> for ParseError<'_> {
    fn from(_: Overflow) -> Self {
        Self::Overflow
    }
}

enum Spec {
    Seconds(u64),
    Minutes(u64),
    Hours(u64),
    Days(u64),
    Weeks(u64),
}

fn parse_spec(s: &str) -> Result<Spec, ParseError> {
    let result: nom::IResult<&str, _> = all_consuming(pair(
        alt((
            recognize(pair(one_of("123456789"), many0(one_of("0123456789")))),
            tag("0"),
        )),
        one_of("smhdw"),
    ))(s);
    let (_, (num, c)) = result?;
    let u = num.parse()?;
    let spec = match c {
        's' => Spec::Seconds(u),
        'm' => Spec::Minutes(u),
        'h' => Spec::Hours(u),
        'd' => Spec::Days(u),
        'w' => Spec::Weeks(u),
        _ => unreachable!(),
    };
    Ok(spec)
}

impl TryFrom<Spec> for Duration {
    type Error = Overflow;

    fn try_from(value: Spec) -> Result<Self, Self::Error> {
        match value {
            Spec::Seconds(u) => Ok(Duration::from_secs(u)),
            Spec::Minutes(u) => {
                Duration::try_from(Spec::Seconds(u.checked_mul(60).ok_or_else(Overflow)?))
            }
            Spec::Hours(u) => {
                Duration::try_from(Spec::Minutes(u.checked_mul(60).ok_or_else(Overflow)?))
            }
            Spec::Days(u) => {
                Duration::try_from(Spec::Hours(u.checked_mul(24).ok_or_else(Overflow)?))
            }
            Spec::Weeks(u) => {
                Duration::try_from(Spec::Days(u.checked_mul(7).ok_or_else(Overflow)?))
            }
        }
    }
}

pub fn parse_duration(s: &str) -> Result<Duration, ParseError> {
    Ok(Duration::try_from(parse_spec(s)?)?)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::parse_duration;

    #[test]
    fn it_parses() {
        assert_eq!(Duration::from_secs(1), parse_duration("1s").unwrap());
        assert_eq!(Duration::from_secs(300), parse_duration("5m").unwrap());
        assert_eq!(Duration::from_secs(43_200), parse_duration("12h").unwrap());
        assert_eq!(Duration::from_secs(172_800), parse_duration("2d").unwrap());
        assert_eq!(
            Duration::from_secs(2_419_200),
            parse_duration("4w").unwrap()
        );
    }
}
