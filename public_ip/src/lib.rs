use std::{
    error::Error as StdError,
    fmt,
    net::{AddrParseError, Ipv4Addr, SocketAddr},
    str::FromStr,
};

use log::{debug, trace};
use reqwest::Response;
use serde_json::Value;
use tokio::net::UdpSocket;
pub use trust_dns_client::rr::Name;
use trust_dns_client::{
    client::{AsyncClient, ClientHandle},
    error::ClientError,
    proto::error::ProtoError,
    rr::{DNSClass, RecordType},
    udp::UdpClientStream,
};
use url::Url;

#[derive(Debug)]
pub enum Error {
    Http(reqwest::Error),
    HttpBadResponse(Response),
    DnsProto(ProtoError),
    DnsClient(ClientError),
    MissingResponse,
    ParseAddr(AddrParseError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Http(e) => e.fmt(f),
            Error::HttpBadResponse(res) => write!(f, "Bad response: {}", res.status()),
            Error::DnsProto(e) => e.fmt(f),
            Error::DnsClient(e) => e.fmt(f),
            Error::MissingResponse => write!(f, "IP not found in response"),
            Error::ParseAddr(e) => e.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Http(e) => Some(e),
            Error::HttpBadResponse(_) => None,
            Error::DnsProto(e) => Some(e),
            Error::DnsClient(e) => Some(e),
            Error::MissingResponse => None,
            Error::ParseAddr(e) => Some(e),
        }
    }
}

impl From<AddrParseError> for Error {
    fn from(e: AddrParseError) -> Self {
        Error::ParseAddr(e)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Http(e)
    }
}

impl From<ProtoError> for Error {
    fn from(e: ProtoError) -> Self {
        Error::DnsProto(e)
    }
}

impl From<ClientError> for Error {
    fn from(e: ClientError) -> Self {
        Error::DnsClient(e)
    }
}

#[derive(Debug)]
pub struct ParseRecordTypeError();

impl fmt::Display for ParseRecordTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unsupported record type")
    }
}

impl StdError for ParseRecordTypeError {}

#[derive(Debug, Eq, PartialEq)]
pub enum DnsRecordType {
    A,
    TXT,
}

impl FromStr for DnsRecordType {
    type Err = ParseRecordTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A" => Ok(DnsRecordType::A),
            "TXT" => Ok(DnsRecordType::TXT),
            _ => Err(ParseRecordTypeError()),
        }
    }
}

impl From<DnsRecordType> for RecordType {
    fn from(val: DnsRecordType) -> Self {
        match val {
            DnsRecordType::A => Self::A,
            DnsRecordType::TXT => Self::TXT,
        }
    }
}

pub enum Service {
    PlainText {
        url: Url,
    },
    Json {
        url: Url,
        key: String,
    },
    Dns {
        server: SocketAddr,
        record_type: RecordType,
        name: Name,
    },
}

impl Service {
    pub fn plain_text(url: Url) -> Service {
        Service::PlainText { url }
    }

    pub fn json(url: Url, key: String) -> Service {
        Service::Json { url, key }
    }

    pub fn dns(server: SocketAddr, record_type: DnsRecordType, name: Name) -> Service {
        Service::Dns {
            server,
            record_type: record_type.into(),
            name,
        }
    }

    pub async fn ipv4(&self) -> Result<Ipv4Addr, Error> {
        match self {
            Service::PlainText { url } => {
                let body = get(url).await?.text().await?;
                trace!("response body: {}", body);
                Ok(body.trim_end().parse()?)
            }
            Service::Json { url, key } => {
                let body = get(url).await?.json::<Value>().await?;
                trace!("response body: {}", body);
                Ok(body
                    .get(&key)
                    .and_then(|v| v.as_str())
                    .ok_or(Error::MissingResponse)?
                    .parse()?)
            }
            Service::Dns {
                server,
                record_type,
                name,
            } => {
                let stream = UdpClientStream::<UdpSocket>::new(server.clone());
                let (mut client, bg) = AsyncClient::connect(stream).await?;
                tokio::spawn(bg);
                debug!("querying {} {} {} {}", server, DNSClass::IN, record_type, &name);
                let mut response = client
                    .query(name.clone(), DNSClass::IN, *record_type)
                    .await?;
                trace!("{:#?}", response);
                let rdata = match response.take_answers().into_iter().next() {
                    Some(a) => a.into_data(),
                    None => return Err(Error::MissingResponse),
                };
                debug!("got result {:?}", rdata);
                let ip = match record_type {
                    RecordType::A => rdata.into_a().expect("expected A record"),
                    RecordType::TXT => rdata
                        .into_txt()
                        .expect("expected TXT record")
                        .to_string()
                        .parse()?,
                    _ => panic!("{} not implemented", record_type),
                };
                Ok(ip)
            }
        }
    }
}

impl Default for Service {
    fn default() -> Service {
        Service::Dns {
            server: ([208, 67, 222, 222], 53).into(), // resolver1.opendns.com
            record_type: RecordType::A,
            name: "myip.opendns.com"
                .parse()
                .expect("hardcoded name shouldn't fail to parse"),
        }
    }
}

async fn get(url: &Url) -> Result<Response, Error> {
    debug!("requesting {}", url);
    let res = reqwest::get(url.clone()).await?;
    debug!("got {} response", res.status());
    if res.status().is_success() {
        Ok(res)
    } else {
        Err(Error::HttpBadResponse(res))
    }
}
