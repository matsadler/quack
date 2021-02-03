use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[derive(Default)]
pub struct UpdateOptions {
    pub(crate) ipv4: Option<Ipv4Addr>,
    pub(crate) ipv6: Option<Ipv6Addr>,
    pub(crate) verbose: bool,
}

impl UpdateOptions {
    pub fn new(ipv4: Ipv4Addr, ipv6: Ipv6Addr, verbose: bool) -> Self {
        Self {
            ipv4: Some(ipv4),
            ipv6: Some(ipv6),
            verbose,
        }
    }

    pub fn ipv4(ipv4: Ipv4Addr, verbose: bool) -> Self {
        Self {
            ipv4: Some(ipv4),
            verbose,
            ..Default::default()
        }
    }

    pub fn ipv6(ipv6: Ipv6Addr, verbose: bool) -> Self {
        Self {
            ipv6: Some(ipv6),
            verbose,
            ..Default::default()
        }
    }

    pub fn verbose() -> Self {
        Self {
            verbose: true,
            ..Default::default()
        }
    }
}

impl From<Ipv4Addr> for UpdateOptions {
    fn from(val: Ipv4Addr) -> Self {
        Self {
            ipv4: Some(val),
            ..Default::default()
        }
    }
}

impl From<Ipv6Addr> for UpdateOptions {
    fn from(val: Ipv6Addr) -> Self {
        Self {
            ipv6: Some(val),
            ..Default::default()
        }
    }
}

impl From<IpAddr> for UpdateOptions {
    fn from(val: IpAddr) -> Self {
        match val {
            IpAddr::V4(v4) => v4.into(),
            IpAddr::V6(v6) => v6.into(),
        }
    }
}

impl From<(Ipv4Addr, Ipv6Addr)> for UpdateOptions {
    fn from(val: (Ipv4Addr, Ipv6Addr)) -> Self {
        Self {
            ipv4: Some(val.0),
            ipv6: Some(val.1),
            ..Default::default()
        }
    }
}

impl From<()> for UpdateOptions {
    fn from(_: ()) -> Self {
        Default::default()
    }
}

#[derive(Default)]
pub struct ClearOptions {
    pub(crate) verbose: bool,
}

impl ClearOptions {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    pub fn verbose() -> Self {
        Self { verbose: true }
    }
}

impl From<()> for ClearOptions {
    fn from(_: ()) -> Self {
        Default::default()
    }
}

pub struct TxtOptions {
    pub(crate) txt: String,
    pub(crate) verbose: bool,
}

impl TxtOptions {
    pub fn new(txt: String, verbose: bool) -> Self {
        Self { txt, verbose }
    }
}

impl From<&str> for TxtOptions {
    fn from(val: &str) -> Self {
        val.to_owned().into()
    }
}

impl From<String> for TxtOptions {
    fn from(val: String) -> Self {
        Self {
            txt: val,
            verbose: false,
        }
    }
}

#[derive(Default)]
pub struct ClearTxtOptions {
    pub(crate) verbose: bool,
}

impl ClearTxtOptions {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    pub fn verbose() -> Self {
        Self { verbose: true }
    }
}

impl From<()> for ClearTxtOptions {
    fn from(_: ()) -> Self {
        Default::default()
    }
}
