mod commands {
    pub mod check_ip;
    pub mod clear;
    pub mod txt;
    pub mod update;
}
mod check_ip_opts;
mod opts;
mod parse_duration;

use std::error::Error as StdError;

use log::{debug, error, info};
use structopt::StructOpt;
use tokio::runtime::Runtime;

use crate::opts::{Command, Opts};

fn main() {
    if let Err(e) = run() {
        error!("{}", e);
        drop(e);
        std::process::exit(1)
    }
}

fn run() -> Result<(), Box<dyn StdError>> {
    let opts = Opts::from_args().propagate_verbose();

    stderrlog::new()
        .modules(vec![module_path!(), "duck_dns", "public_ip"])
        .quiet(opts.quiet)
        .verbosity(opts.verbose)
        .timestamp(stderrlog::Timestamp::Off)
        .color(stderrlog::ColorChoice::Never)
        .show_level(false)
        .show_module_names(false)
        .init()?;

    debug!("{:#?}", opts);

    Runtime::new()?.block_on(async {
        match opts.command {
            Command::Update(c) => {
                let verbose = c.verbose;
                let output = c.run().await?;
                if verbose {
                    println!("{}", output);
                };
            }
            Command::Txt(c) => {
                let verbose = c.verbose;
                let output = c.run().await?;
                if verbose {
                    println!("{}", output);
                };
            }
            Command::Clear(c) => {
                let verbose = c.verbose;
                let output = c.run().await?;
                if verbose {
                    println!("{}", output);
                };
            }
            Command::CheckIp(c) => println!("{}", c.run().await?),
        };
        Ok(())
    })
}
