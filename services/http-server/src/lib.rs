use capwat_kernel::{
  entity::id::{marker::UserMarker, Id},
  services::DataService,
};
use std::sync::Arc;

pub struct App {
  pub data: Arc<dyn DataService>,
}

pub async fn hello(app: App) {
  let user = app
    .data
    .find_user_by_id(Id::<UserMarker>::new(1000))
    .await
    .unwrap()
    .unwrap();
}
