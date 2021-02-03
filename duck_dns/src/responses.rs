use std::{
    error::Error as StdError,
    fmt,
    net::{AddrParseError, Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Status {
    Ok,
    Ko,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Ok => write!(f, "OK"),
            Status::Ko => write!(f, "KO"),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Updated {
    Updated,
    Nochange,
}

impl fmt::Display for Updated {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Updated::Updated => write!(f, "UPDATED"),
            Updated::Nochange => write!(f, "NOCHANGE"),
        }
    }
}

#[derive(Debug)]
pub enum ParseResponseError {
    UnrecognizedStatus,
    BadAddr(AddrParseError),
    UnrecognizedUpdated,
    ExpectedEnd,
}

impl fmt::Display for ParseResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseResponseError::UnrecognizedStatus => write!(f, "unrecognized status line"),
            ParseResponseError::BadAddr(e) => e.fmt(f),
            ParseResponseError::UnrecognizedUpdated => write!(f, "unrecognized updated line"),
            ParseResponseError::ExpectedEnd => write!(f, "expected end of input"),
        }
    }
}

impl From<AddrParseError> for ParseResponseError {
    fn from(e: AddrParseError) -> Self {
        ParseResponseError::BadAddr(e)
    }
}

impl StdError for ParseResponseError {}

#[derive(Debug)]
pub struct Response {
    status: Status,
    ipv4: Option<Ipv4Addr>,
    ipv6: Option<Ipv6Addr>,
    updated: Option<Updated>,
}

impl Response {
    pub fn status(&self) -> Status {
        self.status
    }

    pub fn ipv4(&self) -> Option<&Ipv4Addr> {
        self.ipv4.as_ref()
    }

    pub fn ipv6(&self) -> Option<&Ipv6Addr> {
        self.ipv6.as_ref()
    }

    pub fn updated(&self) -> Option<Updated> {
        self.updated
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.status)?;
        if let Some(updated) = self.updated {
            writeln!(f)?;
            match self.ipv4 {
                Some(ipv4) => writeln!(f, "{}", ipv4)?,
                None => writeln!(f)?,
            }
            match self.ipv6 {
                Some(ipv6) => writeln!(f, "{}", ipv6)?,
                None => writeln!(f)?,
            }
            write!(f, "{}", updated)?;
        }
        Ok(())
    }
}

impl FromStr for Response {
    type Err = ParseResponseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let status = match lines.next() {
            Some("OK") => Status::Ok,
            Some("KO") => Status::Ko,
            _ => return Err(ParseResponseError::UnrecognizedStatus),
        };
        let ipv4 = match lines.next() {
            Some("") => None,
            Some(ipv4) => Some(ipv4.parse()?),
            None => None,
        };
        let ipv6 = match lines.next() {
            Some("") => None,
            Some(ipv6) => Some(ipv6.parse()?),
            None => None,
        };
        let updated = match lines.next() {
            Some("UPDATED") => Some(Updated::Updated),
            Some("NOCHANGE") => Some(Updated::Nochange),
            Some(_) => return Err(ParseResponseError::UnrecognizedUpdated),
            None => None,
        };
        let last = lines.next();
        if !(last == None || (last == Some("") && lines.next() == None)) {
            return Err(ParseResponseError::ExpectedEnd);
        }
        Ok(Response {
            status,
            ipv4,
            ipv6,
            updated,
        })
    }
}

#[derive(Debug)]
pub enum ParseTxtResponseError {
    UnrecognizedStatus,
    UnrecognizedUpdated,
    ExpectedEnd,
}

impl fmt::Display for ParseTxtResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseTxtResponseError::UnrecognizedStatus => write!(f, "unrecognized status line"),
            ParseTxtResponseError::UnrecognizedUpdated => write!(f, "unrecognized updated line"),
            ParseTxtResponseError::ExpectedEnd => write!(f, "expected end of input"),
        }
    }
}

impl StdError for ParseTxtResponseError {}

#[derive(Debug)]
pub struct TxtResponse {
    status: Status,
    txt: Option<String>,
    updated: Option<Updated>,
}

impl TxtResponse {
    pub fn status(&self) -> Status {
        self.status
    }

    pub fn txt(&self) -> Option<&str> {
        self.txt.as_deref()
    }

    pub fn updated(&self) -> Option<Updated> {
        self.updated
    }
}

impl fmt::Display for TxtResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.status)?;
        if let Some(updated) = self.updated {
            writeln!(f)?;
            match self.txt {
                Some(ref txt) => writeln!(f, "{}", txt)?,
                None => writeln!(f)?,
            }
            write!(f, "{}", updated)?;
        }
        Ok(())
    }
}

impl FromStr for TxtResponse {
    type Err = ParseTxtResponseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let status = match lines.next() {
            Some("OK") => Status::Ok,
            Some("KO") => Status::Ko,
            _ => return Err(ParseTxtResponseError::UnrecognizedStatus),
        };
        let txt = match lines.next() {
            Some(txt) => Some(txt.to_owned()),
            None => None,
        };
        let updated = match lines.next() {
            Some("UPDATED") => Some(Updated::Updated),
            Some("NOCHANGE") => Some(Updated::Nochange),
            Some(_) => return Err(ParseTxtResponseError::UnrecognizedUpdated),
            None => None,
        };
        let last = lines.next();
        if !(last == None || (last == Some("") && lines.next() == None)) {
            return Err(ParseTxtResponseError::ExpectedEnd);
        }
        Ok(TxtResponse {
            status,
            txt,
            updated,
        })
    }
}
