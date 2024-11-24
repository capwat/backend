use capwat_config::Server as Config;
use capwat_error::{ext::ResultExt, Result};
use capwat_server::StartServerError;
use capwat_utils::env::load_dotenv;
use capwat_vfs::Vfs;

use clap::Parser;
use std::net::IpAddr;
use std::num::NonZeroUsize;

/// Expose a Capwat API HTTP server
#[derive(Debug, Parser)]
pub struct ServerCommand {
    #[clap(long)]
    pub address: Option<IpAddr>,
    #[clap(long)]
    pub port: Option<u16>,
    #[clap(long)]
    pub workers: Option<NonZeroUsize>,
}

pub fn run(args: ServerCommand) -> Result<(), StartServerError> {
    let vfs = Vfs::new_std();
    load_dotenv(&vfs).ok();

    let mut config = Config::from_maybe_file(&vfs).change_context(StartServerError)?;
    args.override_config(&mut config);

    let _guard = capwat_tracing::init(&config.logging).change_context(StartServerError)?;
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(config.workers)
        .build()
        .change_context(StartServerError)
        .attach_printable("could not build tokio runtime")?
        .block_on(capwat_server::run(config))
}

impl ServerCommand {
    fn override_config(&self, config: &mut Config) {
        // override server configurations if set by the cli
        if let Some(address) = self.address {
            config.ip = address;
        }

        if let Some(port) = self.port {
            config.port = port;
        }

        if let Some(workers) = self.workers {
            config.workers = workers.get();
        }
    }
}
