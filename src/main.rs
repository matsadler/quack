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

use log::{debug, error};
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

    let mut logger = stderrlog::new();
    // most of the time we want to scope logging to just this codebase, but
    // extra verbose -vvvvv level turns on logging for all modules, but only
    // in debug builds (it might leak tokens, so isn't safe for release builds)
    if opts.verbose < 5 || !cfg!(debug_assertions) {
        logger.modules(vec![module_path!(), "duck_dns", "public_ip"]);
    }
    logger.quiet(opts.quiet)
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
