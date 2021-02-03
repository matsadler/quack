use std::{error::Error as StdError, net::Ipv4Addr};

use structopt::StructOpt;

use crate::check_ip_opts::CheckIpOpts;

#[derive(StructOpt, Debug)]
pub struct CheckIp {
    #[structopt(short, long)]
    pub opts: Option<CheckIpOpts>,
}

impl CheckIp {
    pub async fn run(self) -> Result<Ipv4Addr, Box<dyn StdError>> {
        let service = match self.opts {
            Some(opts) => opts.into_service().await?,
            None => Default::default(),
        };
        Ok(service.ipv4().await?)
    }
}
