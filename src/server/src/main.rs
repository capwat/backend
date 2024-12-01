use capwat_crypto::client::generate_mock_user_keys;
use capwat_error::{ext::ResultExt, Result};
use capwat_model::instance_settings::InstanceSettings;
use capwat_postgres::queries::instance_settings::InstanceSettingsPgImpl;
use capwat_server::App;
use capwat_utils::env::load_dotenv;
use capwat_vfs::Vfs;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{info, warn};

fn install_all_error_middlewares() {
    capwat_postgres::install_error_middleware();
}

async fn setup_instance(app: App) -> Result<()> {
    let mut conn = app.db_write().await?;
    InstanceSettings::setup_local(&mut conn).await?;
    conn.commit().await?;

    let mut conn = app.db_read().await?;
    let settings = InstanceSettings::get_local(&mut conn).await?;
    if settings.require_captcha && app.config.hcaptcha.is_none() {
        warn!("hCaptcha integration is not configured but the instance settings requires CAPTCHA. Please configure hCaptcha or turn off `Require CAPTCHA` in instance settings.");
    }

    Ok(())
}

async fn stuff(config: capwat_config::Server, vfs: Vfs) -> Result<()> {
    let data = generate_mock_user_keys("memothelemo");
    println!("{data:#?}");

    let app = App::new(config, vfs);
    tokio::spawn({
        let app = app.clone();
        async move {
            if let Err(error) = setup_instance(app).await {
                warn!(%error, "could not setup instance");
            }
        }
    });

    let listener = TcpListener::bind((app.config.ip, app.config.port))
        .await
        .attach_printable("could not bind server with address and port")?;

    let addr = listener
        .local_addr()
        .attach_printable("could not get socket address of the server")?;

    let router = capwat_server::controllers::build_axum_router(app.clone());
    let router = capwat_server::middleware::apply(router);

    info!(
        "Confessions server is listening at http://{addr} with {} workers",
        app.config.workers
    );

    let make_service = router.into_make_service_with_connect_info::<SocketAddr>();
    axum::serve(listener, make_service)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

#[cfg(not(unix))]
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install ctrl+c handler");
}

#[cfg(unix)]
async fn shutdown_signal() {
    use tokio::signal::unix::{signal, SignalKind};

    let interrupt = async {
        signal(SignalKind::interrupt())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    let terminate = async {
        signal(SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = interrupt => {},
        _ = terminate => {},
    }
}

fn start() -> Result<()> {
    let vfs = Vfs::new_std();
    load_dotenv(&vfs).ok();

    let config = capwat_config::Server::from_maybe_file(&vfs)?;
    capwat_tracing::init(&config.logging)?;

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(stuff(config, vfs))
}

fn main() -> std::process::ExitCode {
    install_all_error_middlewares();

    if let Err(error) = start() {
        eprintln!("{error:#}");
        std::process::ExitCode::FAILURE
    } else {
        std::process::ExitCode::SUCCESS
    }
}
