use clap::Parser;
use std::process;

mod cli;

fn main() {
    let cli = self::cli::Cli::parse();
    if let Err(error) = cli.run() {
        eprintln!("{error:#}");
        process::exit(1);
    }
}
