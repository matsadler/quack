use std::error::Error as StdError;

use duck_dns::{Client, TxtOptions};
use structopt::StructOpt;

use crate::opts::Account;

#[derive(StructOpt, Debug)]
pub struct Txt {
    #[structopt(short = "x", long)]
    pub txt: String,
    #[structopt(flatten)]
    pub account: Account,
    #[structopt(skip)]
    pub verbose: bool,
}

impl Txt {
    pub async fn run(self) -> Result<duck_dns::TxtResponse, Box<dyn StdError>> {
        let client = Client::from(self.account);
        Ok(client
            .update_txt(TxtOptions::new(self.txt, self.verbose))
            .await?)
    }
}
