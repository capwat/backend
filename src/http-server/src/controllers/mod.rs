use actix_web::web;

pub mod users;

pub fn configure(cfg: &mut web::ServiceConfig) {
  cfg.service(web::scope("/users").route("/register", web::get().to(users::register)));
}
