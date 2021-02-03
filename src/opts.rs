use duck_dns::{Label, Token};
use structopt::StructOpt;

use crate::commands::{check_ip::CheckIp, clear::Clear, txt::Txt, update::Update};

#[derive(StructOpt, Debug)]
pub struct Opts {
    /// Silence all output
    #[structopt(short = "q", long = "quiet", conflicts_with = "verbose")]
    pub quiet: bool,
    /// Verbose mode, multiples increase the verbosity
    #[structopt(short, long, global = true, parse(from_occurrences))]
    pub verbose: usize,
    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    Update(Update),
    Txt(Txt),
    Clear(Clear),
    CheckIp(CheckIp),
}

#[derive(StructOpt, Debug)]
pub struct Account {
    #[structopt(short, long, parse(from_str), env = "DUCKDNS_TOKEN")]
    pub token: Token,
    #[structopt(required = true)]
    pub domain: Vec<Label>,
}

impl From<Account> for duck_dns::Client {
    fn from(value: Account) -> Self {
        Self::new(value.domain, value.token)
    }
}

impl Opts {
    pub fn propagate_verbose(mut self) -> Self {
        if self.verbose == 0 {
            return self;
        }
        match self.command {
            Command::Update(ref mut c) => c.verbose = true,
            Command::Txt(ref mut c) => c.verbose = true,
            Command::Clear(ref mut c) => c.verbose = true,
            Command::CheckIp(_) => (),
        };
        self
    }
}
