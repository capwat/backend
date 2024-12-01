use crate::App;
use axum_test::TestServer;
use capwat_model::instance_settings::InstanceSettings;
use capwat_postgres::queries::instance_settings::InstanceSettingsPgImpl;
use capwat_vfs::{backend::InMemoryFs, Vfs};
use tracing::info;

pub async fn build_test_server() -> (TestServer, App) {
    let vfs = Vfs::new(InMemoryFs::new());
    let _ = capwat_utils::env::load_dotenv(&Vfs::new_std());

    capwat_tracing::init_for_tests();
    capwat_postgres::install_error_middleware();

    let app = App::new_for_tests(vfs).await;

    info!("setting up local instance settings");
    let mut conn = app.db_write().await.unwrap();
    InstanceSettings::setup_local(&mut conn).await.unwrap();
    conn.commit().await.unwrap();

    info!("initializing test server");
    let router = crate::controllers::build_axum_router(app.clone());
    let router = crate::middleware::apply(router);
    (TestServer::new(router).unwrap(), app)
}
