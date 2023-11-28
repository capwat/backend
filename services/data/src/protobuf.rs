mod data {
  tonic::include_proto!("data");
}
mod schema {
  tonic::include_proto!("schema");
}
pub use data::*;
pub use schema::User;
