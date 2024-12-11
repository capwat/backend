use capwat_error::{ext::ResultExt, Result};
use capwat_model::instance::InstanceSettings;
use capwat_server::App;
use capwat_utils::{env::load_dotenv, future::Retry};
use capwat_vfs::Vfs;
use futures::TryFutureExt;
use std::{net::SocketAddr, time::Duration};
use thiserror::Error;
use tokio::net::TcpListener;
use tracing::{debug, info, warn, Instrument};

#[derive(Debug, Error)]
#[error("Could not start Capwat HTTP server")]
struct StartError;

#[tracing::instrument(skip_all, name = "server.run")]
async fn start_capwat_server(config: capwat_config::Server, vfs: Vfs) -> Result<(), StartError> {
    if !capwat_utils::RELEASE {
        info!(?config, "Starting Capwat HTTP server...");
    }

    // Setup the entire instance separately on a different thread...
    let app = App::new(config, vfs).change_context(StartError)?;
    tokio::spawn({
        let app = app.clone();
        let span = tracing::info_span!("instance.setup");
        setup_instance(app)
            .instrument(span.clone())
            .inspect_err(move |error| {
                span.in_scope(|| {
                    warn!(%error, "Could not setup Capwat instance settings");
                })
            })
    });

    debug!("binding server");
    let listener = TcpListener::bind((app.config.ip, app.config.port))
        .await
        .change_context(StartError)
        .attach_printable("could not bind server with address and port")?;

    let addr = listener
        .local_addr()
        .change_context(StartError)
        .attach_printable("could not get socket address of the server")?;

    let make_service = capwat_server::build_axum_router(app.clone())
        .into_make_service_with_connect_info::<SocketAddr>();

    info!(
        "Capwat HTTP server is listening at http://{addr} with {} workers",
        app.config.workers
    );

    axum::serve(listener, make_service)
        .with_graceful_shutdown(
            async {
                capwat_utils::shutdown_signal().await;
                info!("Received graceful shutdown signal. Shutting down server...");
            }
            .instrument(tracing::Span::current()),
        )
        .await
        .change_context(StartError)
        .attach_printable("could not serve Capwat HTTP service")?;

    Ok(())
}

async fn setup_instance(app: App) -> Result<()> {
    debug!("setting up Capwat instance settings...");

    Retry::builder("Setup Capwat instance", || async {
        let mut conn = app.db_write().await?;
        InstanceSettings::setup_local(&mut conn).await?;

        let settings = InstanceSettings::get_local(&mut conn).await?;
        if settings.require_captcha && app.config.hcaptcha.is_none() {
            warn!("hCaptcha integration is not configured but the instance settings requires CAPTCHA. Please configure hCaptcha or turn off `Require CAPTCHA` in instance settings.");
        }
        conn.commit().await?;

        Ok::<_, capwat_error::Error>(())
    })
    .max_retries(3)
    // 30 seconds retry so we don't have to spam database
    // operations too quickly.
    .wait(Duration::from_secs(30))
    .build()
    .run()
    .await?;

    Ok(())
}

#[capwat_macros::main]
fn main() -> Result<(), StartError> {
    capwat_db::install_error_middleware();

    let vfs = Vfs::new_std();
    load_dotenv(&vfs).ok();

    let config = capwat_config::Server::from_maybe_file(&vfs).change_context(StartError)?;
    capwat_tracing::init(&config.logging).change_context(StartError)?;

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(config.workers)
        .build()
        .unwrap();

    rt.block_on(start_capwat_server(config, vfs))
}
