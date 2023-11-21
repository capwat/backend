use actix_web::web;

pub mod users;

pub fn configure(cfg: &mut web::ServiceConfig) {
  cfg.service(
    web::scope("/users")
      .service(web::resource("/@{name}").route(web::get().to(users::profile)))
      .route("/login", web::post().to(users::login))
      .route("/register", web::post().to(users::register)),
  );
}
