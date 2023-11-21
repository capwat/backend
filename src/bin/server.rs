use actix_web::{middleware::ErrorHandlers, web, App, HttpServer};
use tracing_actix_web::TracingLogger;
use whim::config;

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt()
    .pretty()
    .with_max_level(tracing::Level::DEBUG)
    .init();

  let config = config::Server::from_env().unwrap();
  let app = whim::App::new(config).await.unwrap();

  HttpServer::new(move || {
    App::new()
      .app_data(web::Data::new(app.clone()))
      .wrap(TracingLogger::<whim::http::util::QuieterRootSpanBuilder>::new())
      .wrap(ErrorHandlers::new().default_handler(whim::http::util::handle_actix_web_error))
      .configure(whim::http::controllers::configure)
  })
  .workers(1)
  .bind(("localhost", 3000))
  .unwrap()
  .run()
  .await
  .unwrap();
}
