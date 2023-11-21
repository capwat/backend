use actix_web::{web, HttpServer};
use whim_core::config;
use whim_http_server::App;

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt()
    .pretty()
    .with_max_level(tracing::Level::DEBUG)
    .init();

  let config = config::Server::from_env().unwrap();
  let app = App::new(config).await.unwrap();

  HttpServer::new(move || {
    actix_web::App::new()
      .app_data(web::Data::new(app.clone()))
      .configure(whim_http_server::controllers::configure)
  })
  .workers(1)
  .bind(("localhost", 3000))
  .unwrap()
  .run()
  .await
  .unwrap();
}
