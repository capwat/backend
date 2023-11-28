use capwat_data_service::ClientLayer;
use capwat_kernel::{
  entity::id::{marker::UserMarker, Id},
  services::DataService,
};

pub struct App {
  pub data: ClientLayer,
}

pub async fn hello(app: App) {
  let user = app
    .data
    .find_user_by_id(Id::<UserMarker>::new(1000))
    .await
    .unwrap()
    .unwrap();
}
