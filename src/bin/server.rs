use actix_web::{web, App, HttpServer};
use whim::config;

fn routing(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api").service(
            web::scope("/v1").service(
                web::scope("/users")
                    .route(
                        "/login",
                        web::post().to(whim::controllers::users::login::post),
                    )
                    .route(
                        "/register",
                        web::post().to(whim::controllers::users::register::post),
                    )
                    .route("/@me", web::get().to(whim::controllers::users::me::get)),
            ),
        ),
    );
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .pretty()
        .init();

    let config = config::Server::load().unwrap();
    let app = whim::App::new(config).await.unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app.clone()))
            .configure(routing)
    })
    .workers(1)
    .bind(("localhost", 3000))
    .unwrap()
    .run()
    .await
    .unwrap();
}
