use capwat_error::{ext::ResultExt, Result};
use clap::Parser;

mod server;

/// Command line options for Capwat.
#[derive(Debug, Parser)]
#[command(
    about = "Utility suite for Capwat backend",
    version,
    author,
    long_about
)]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommand: Subcommand,
}

impl Cli {
    pub fn run(self) -> Result<()> {
        match self.subcommand {
            Subcommand::Server(args) => self::server::run(args).erase_context(),
        }
    }
}

#[derive(Debug, Parser)]
pub enum Subcommand {
    Server(self::server::ServerCommand),
}
