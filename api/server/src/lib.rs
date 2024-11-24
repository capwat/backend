use capwat_error::{ext::ResultExt, Result};
use thiserror::Error;
use tokio::net::TcpListener;
use tracing::{debug, info};

#[derive(Debug, Error)]
#[error("Could not start the Capwat server")]
pub struct StartServerError;

#[tracing::instrument(skip_all, name = "server.run", fields(
    server.ip = %config.ip,
    server.port = %config.port,
    workers = %config.workers,
))]
pub async fn run(config: capwat_config::Server) -> Result<(), StartServerError> {
    if capwat_utils::RELEASE {
        debug!("Starting server...");
    } else {
        info!("Starting server with config: {config:#?}");
    }

    let listener = TcpListener::bind((config.ip, config.port))
        .await
        .change_context(StartServerError)
        .attach_printable("could not bind server with address and port")?;

    let addr = listener
        .local_addr()
        .change_context(StartServerError)
        .attach_printable("could not get socket address of the server")?;

    info!("Capwat server is listening at http://{addr}");

    Ok(())
}
