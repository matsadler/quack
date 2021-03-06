use std::{
    error::Error as StdError,
    fmt,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    time::Duration,
};

use duck_dns::{Client, UpdateOptions};
use log::{debug, error, info};
use structopt::StructOpt;

use crate::{check_ip_opts::CheckIpOpts, opts::Account, parse_duration::parse_duration};

#[derive(Debug)]
struct IpOptError();

impl fmt::Display for IpOptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ip cannot be a v6 ip when ipv6 is also present")
    }
}

impl StdError for IpOptError {}

#[derive(StructOpt, Debug)]
pub struct Update {
    #[structopt(short, long)]
    pub ip: Option<IpAddr>,
    #[structopt(short = "6", long)]
    pub ipv6: Option<Ipv6Addr>,
    #[structopt(short, long, conflicts_with_all = &["ip", "ipv6"])]
    pub preflight_ip: bool,
    #[structopt(short = "o", long, conflicts_with_all = &["ip", "ipv6"])]
    pub preflight_opts: Option<CheckIpOpts>,
    #[structopt(short, long, parse(try_from_str = parse_duration), conflicts_with_all = &["ip", "ipv6"])]
    pub schedule: Option<Duration>,
    #[structopt(flatten)]
    pub account: Account,
    #[structopt(skip)]
    pub verbose: bool,
}

impl Update {
    pub async fn run(self) -> Result<duck_dns::Response, Box<dyn StdError>> {
        let schedule = match self.schedule {
            Some(schedule) => schedule,
            None => return update_now(self).await,
        };

        let client = Client::from(self.account);
        let service = match self.preflight_opts {
            Some(opts) => Some(opts.into_service().await?),
            None if self.preflight_ip => Some(Default::default()),
            None => None,
        };

        let mut prev_ip = Ipv4Addr::UNSPECIFIED;

        loop {
            if let Some(ref service) = service {
                match update_preflight_schedule(&client, service, prev_ip, self.verbose).await {
                    Ok((res, ip)) => {
                        debug!("prev_ip = {}, ip = {}", prev_ip, ip);
                        prev_ip = ip;
                        match res {
                            Some(r) => info!("{}", r),
                            None => info!("no ip change, skipping update"),
                        }
                    }
                    Err(e) => {
                        error!("error during IP preflight: {}", e);
                        let d = schedule.min(Duration::from_secs(60));
                        debug!("sleeping for {:?}", d);
                        tokio::time::sleep(d).await;
                        continue;
                    }
                };
            } else {
                match update_schedule(&client, self.verbose).await {
                    Ok(r) => info!("{}", r),
                    Err(e) => error!("{}", e),
                };
            }

            debug!("sleeping for {:?}", schedule);
            tokio::time::sleep(schedule).await
        }
    }
}

async fn update_now(opts: Update) -> Result<duck_dns::Response, Box<dyn StdError>> {
    let client = Client::from(opts.account);

    let args = match (opts.ip, opts.ipv6) {
        (Some(IpAddr::V4(ip)), None) => UpdateOptions::ipv4(ip, opts.verbose),
        (Some(IpAddr::V6(ipv6)), None) | (None, Some(ipv6)) => {
            UpdateOptions::ipv6(ipv6, opts.verbose)
        }
        (Some(IpAddr::V4(ip)), Some(ipv6)) => UpdateOptions::new(ip, ipv6, opts.verbose),
        (Some(IpAddr::V6(_)), Some(_)) => return Err(IpOptError().into()),
        (None, None) if opts.preflight_ip || opts.preflight_opts.is_some() => {
            let service = match opts.preflight_opts {
                Some(opts) => opts.into_service().await?,
                None => Default::default(),
            };
            let ip = service.ipv4().await?;
            UpdateOptions::ipv4(ip, opts.verbose)
        }
        (None, None) if opts.verbose => UpdateOptions::verbose(),
        (None, None) => UpdateOptions::default(),
    };

    Ok(client.update(args).await?)
}

async fn update_preflight_schedule(
    client: &Client,
    service: &public_ip::Service,
    prev_ip: Ipv4Addr,
    verbose: bool,
) -> Result<(Option<duck_dns::Response>, Ipv4Addr), Box<dyn StdError>> {
    let ip = service.ipv4().await?;
    if ip == prev_ip {
        return Ok((None, prev_ip));
    }
    let args = UpdateOptions::ipv4(ip, verbose);
    let response = client.update(args).await?;
    Ok((Some(response), ip))
}

async fn update_schedule(
    client: &Client,
    verbose: bool,
) -> Result<duck_dns::Response, Box<dyn StdError>> {
    let args = if verbose {
        UpdateOptions::verbose()
    } else {
        UpdateOptions::default()
    };
    Ok(client.update(args).await?)
}
