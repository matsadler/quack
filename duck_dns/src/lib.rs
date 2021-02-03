mod label;
mod options;
mod responses;

use std::{error::Error as StdError, fmt, str::FromStr};

use log::debug;
use url::Url;

pub use crate::{label::*, options::*, responses::*};

pub struct Token(String);

impl Token {
    fn scrub(&self, s: &str) -> String {
        s.replace(&self.0, &"*".repeat(self.0.len()))
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Token")
            .field(&"*".repeat(self.0.len()))
            .finish()
    }
}

impl From<String> for Token {
    fn from(val: String) -> Self {
        Token(val)
    }
}

impl From<&str> for Token {
    fn from(val: &str) -> Self {
        Token(val.to_owned())
    }
}

#[derive(Debug)]
pub enum Error {
    Http(reqwest::Error),
    HttpBadResponse(reqwest::Response),
    ParseResponse(ParseResponseError),
    ParseTxtResponse(ParseTxtResponseError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Http(e) => e.fmt(f),
            Error::HttpBadResponse(res) => write!(f, "bad response: {}", res.status()),
            Error::ParseResponse(e) => e.fmt(f),
            Error::ParseTxtResponse(e) => e.fmt(f),
        }
    }
}

impl StdError for Error {}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Http(e)
    }
}

impl From<ParseResponseError> for Error {
    fn from(e: ParseResponseError) -> Self {
        Error::ParseResponse(e)
    }
}

impl From<ParseTxtResponseError> for Error {
    fn from(e: ParseTxtResponseError) -> Self {
        Error::ParseTxtResponse(e)
    }
}

pub struct Client {
    url: Url,
    domains: Vec<Label>,
    token: Token,
}

impl Client {
    pub fn new<T>(domains: Vec<Label>, token: T) -> Self
    where
        T: Into<Token>,
    {
        let url = Url::parse("https://www.duckdns.org/update")
            .expect("hardcoded URL shouldn't fail to parse");
        Self::with_base(url, domains, token)
    }

    pub fn with_base<T>(url: Url, domains: Vec<Label>, token: T) -> Self
    where
        T: Into<Token>,
    {
        Self {
            url,
            domains,
            token: token.into(),
        }
    }

    pub async fn update<T>(&self, options: T) -> Result<Response, Error>
    where
        T: Into<UpdateOptions>,
    {
        let options = options.into();
        let mut url = self.url.clone();
        {
            let mut query = url.query_pairs_mut();
            if let Some(ipv4) = options.ipv4 {
                query.append_pair("ip", &ipv4.to_string());
            }
            if let Some(ipv6) = options.ipv6 {
                query.append_pair("ipv6", &ipv6.to_string());
            }
            if options.verbose {
                query.append_pair("verbose", "true");
            }
        }
        self.request(url).await
    }

    pub async fn clear<T>(&self, options: T) -> Result<Response, Error>
    where
        T: Into<ClearOptions>,
    {
        let options = options.into();
        let mut url = self.url.clone();
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("clear", "true");
            if options.verbose {
                query.append_pair("verbose", "true");
            }
        }
        self.request(url).await
    }

    pub async fn update_txt<T>(&self, options: T) -> Result<TxtResponse, Error>
    where
        T: Into<TxtOptions>,
    {
        let options = options.into();
        let mut url = self.url.clone();
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("txt", &options.txt);
            if options.verbose {
                query.append_pair("verbose", "true");
            }
        }
        self.request(url).await
    }

    pub async fn clear_txt<T>(&self, options: T) -> Result<TxtResponse, Error>
    where
        T: Into<ClearTxtOptions>,
    {
        let options = options.into();
        let mut url = self.url.clone();
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("txt", "");
            query.append_pair("clear", "true");
            if options.verbose {
                query.append_pair("verbose", "true");
            }
        }
        self.request(url).await
    }

    async fn request<T>(&self, mut url: Url) -> Result<T, Error>
    where
        T: FromStr,
        Error: From<<T as FromStr>::Err>,
    {
        {
            let mut query = url.query_pairs_mut();
            query.append_pair(
                "domains",
                &self
                    .domains
                    .iter()
                    .map(|d| d.0.as_ref())
                    .collect::<Vec<_>>()
                    .join(","),
            );
            query.append_pair("token", &self.token.0);
        }
        debug!("requesting {}", self.token.scrub(url.as_str()));
        let res = reqwest::get(url).await?;
        debug!("got {} response", res.status());
        if res.status().is_success() {
            Ok(res.text().await?.parse()?)
        } else {
            Err(Error::HttpBadResponse(res))
        }
    }
}
