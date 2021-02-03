use std::{error::Error as StdError, fmt::Display};

use duck_dns::{ClearOptions, ClearTxtOptions, Client};
use structopt::StructOpt;

use crate::opts::Account;

#[derive(StructOpt, Debug)]
pub struct Clear {
    #[structopt(short = "x", long)]
    pub txt: bool,
    #[structopt(flatten)]
    pub account: Account,
    #[structopt(skip)]
    pub verbose: bool,
}

impl Clear {
    pub async fn run(self) -> Result<Box<dyn Display>, Box<dyn StdError>> {
        let client = Client::from(self.account);
        if self.txt {
            Ok(Box::new(
                client.clear_txt(ClearTxtOptions::new(self.verbose)).await?,
            ))
        } else {
            Ok(Box::new(
                client.clear(ClearOptions::new(self.verbose)).await?,
            ))
        }
    }
}
