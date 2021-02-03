use std::{
    error::Error as StdError,
    fmt, io,
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{space0, space1},
    combinator::{all_consuming, map, map_res, opt, rest},
    sequence::{preceded, terminated, tuple},
    IResult,
};
use public_ip::{DnsRecordType, Name};
use tokio::net::lookup_host;
use url::Url;

#[derive(Debug)]
pub struct ParseServerError();

#[derive(Debug, PartialEq)]
pub enum Server {
    Host(Name),
    Ip(IpAddr),
    SocketAddr(SocketAddr),
}

impl Server {
    async fn into_socket_addr(self) -> Result<SocketAddr, io::Error> {
        match self {
            Server::Host(name) => lookup_host((name.to_string().as_ref(), 53))
                .await?
                .next()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "could not resolve host")),
            Server::Ip(ip) => Ok((ip, 53).into()),
            Server::SocketAddr(addr) => Ok(addr),
        }
    }
}

impl FromStr for Server {
    type Err = ParseServerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse() {
            Ok(v) => Ok(Server::SocketAddr(v)),
            Err(_) => match s.parse() {
                Ok(v) => Ok(Server::Ip(v)),
                Err(_) => match s.parse() {
                    Ok(v) => Ok(Server::Host(v)),
                    Err(_) => Err(ParseServerError()),
                },
            },
        }
    }
}

#[derive(Debug)]
pub struct ParseIpOptsError();

impl fmt::Display for ParseIpOptsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bad options format")
    }
}

impl StdError for ParseIpOptsError {}

#[derive(Debug, PartialEq)]
pub enum CheckIpOpts {
    PlainText {
        url: Url,
    },
    Json {
        url: Url,
        key: String,
    },
    Dns {
        server: Server,
        record_type: Option<DnsRecordType>,
        name: Name,
    },
}

impl CheckIpOpts {
    pub async fn into_service(self) -> Result<public_ip::Service, io::Error> {
        match self {
            CheckIpOpts::PlainText { url } => Ok(public_ip::Service::plain_text(url)),
            CheckIpOpts::Json { url, key } => Ok(public_ip::Service::json(url, key)),
            CheckIpOpts::Dns {
                server,
                record_type,
                name,
            } => server.into_socket_addr().await.map(|server| {
                public_ip::Service::dns(server, record_type.unwrap_or(DnsRecordType::A), name)
            }),
        }
    }
}

impl FromStr for CheckIpOpts {
    type Err = ParseIpOptsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match ip_opts(s) {
            Ok((_, opts)) => Ok(opts),
            Err(_) => Err(ParseIpOptsError()),
        }
    }
}

fn ip_opts(i: &str) -> IResult<&str, CheckIpOpts> {
    all_consuming(alt((
        map(json, |(key, url)| CheckIpOpts::Json {
            url,
            key: key.to_owned(),
        }),
        map(dns, |(server, record_type, name)| CheckIpOpts::Dns {
            server,
            record_type,
            name,
        }),
        map(url, |url| CheckIpOpts::PlainText { url }),
    )))(i)
}

fn url(i: &str) -> IResult<&str, Url> {
    map_res(rest, |i: &str| i.parse())(i)
}

fn json(i: &str) -> IResult<&str, (&str, Url)> {
    preceded(
        terminated(tag("json:"), space0),
        tuple((terminated(is_not(" \t"), space1), url)),
    )(i)
}

fn dns(i: &str) -> IResult<&str, (Server, Option<DnsRecordType>, Name)> {
    tuple((
        preceded(
            tag("@"),
            terminated(map_res(is_not(" \t"), |i: &str| i.parse()), space1),
        ),
        opt(terminated(
            map_res(is_not(" \t"), |i: &str| i.parse()),
            space1,
        )),
        map_res(rest, |i: &str| i.parse()),
    ))(i)
}

#[cfg(test)]
mod tests {
    use public_ip::DnsRecordType;

    use super::{CheckIpOpts, Server};

    #[test]
    fn it_parses_url_as_plain_text() {
        assert_eq!(
            "http://example.com/".parse::<CheckIpOpts>().unwrap(),
            CheckIpOpts::PlainText {
                url: "http://example.com/".parse().unwrap()
            },
        );
    }

    #[test]
    fn it_parses_json() {
        assert_eq!(
            "json:ip http://example.com/"
                .parse::<CheckIpOpts>()
                .unwrap(),
            CheckIpOpts::Json {
                url: "http://example.com/".parse().unwrap(),
                key: "ip".into(),
            },
        );
    }

    #[test]
    fn it_parses_json_with_space() {
        assert_eq!(
            "json: ip http://example.com/"
                .parse::<CheckIpOpts>()
                .unwrap(),
            CheckIpOpts::Json {
                url: "http://example.com/".parse().unwrap(),
                key: "ip".into(),
            },
        );
    }

    #[test]
    fn it_parses_dns_host() {
        assert_eq!(
            "@example.com A ip.example.com"
                .parse::<CheckIpOpts>()
                .unwrap(),
            CheckIpOpts::Dns {
                server: Server::Host("example.com".parse().unwrap()),
                record_type: Some(DnsRecordType::A),
                name: "ip.example.com".parse().unwrap(),
            },
        );
    }

    #[test]
    fn it_parses_dns_ip() {
        assert_eq!(
            "@192.0.2.1 A ip.example.com"
                .parse::<CheckIpOpts>()
                .unwrap(),
            CheckIpOpts::Dns {
                server: Server::Ip("192.0.2.1".parse().unwrap()),
                record_type: Some(DnsRecordType::A),
                name: "ip.example.com".parse().unwrap(),
            },
        );
    }

    #[test]
    fn it_parses_dns_socket_addr() {
        assert_eq!(
            "@192.0.2.1:53 A ip.example.com"
                .parse::<CheckIpOpts>()
                .unwrap(),
            CheckIpOpts::Dns {
                server: Server::SocketAddr("192.0.2.1:53".parse().unwrap()),
                record_type: Some(DnsRecordType::A),
                name: "ip.example.com".parse().unwrap(),
            },
        );
    }

    #[test]
    fn it_parses_dns_without_record_type() {
        assert_eq!(
            "@example.com ip.example.com"
                .parse::<CheckIpOpts>()
                .unwrap(),
            CheckIpOpts::Dns {
                server: Server::Host("example.com".parse().unwrap()),
                record_type: None,
                name: "ip.example.com".parse().unwrap(),
            },
        );
    }
}
