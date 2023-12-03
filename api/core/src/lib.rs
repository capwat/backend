use async_trait::async_trait;
use capwat_kernel::drivers::{data::types, prelude::*};

mod app;
pub use app::App;

#[async_trait]
pub trait Perform {
  type Response;

  async fn perform(&self, app: &App) -> KResult<Self::Response>;
}

#[async_trait]
impl Perform for capwat_types::forms::Register {
  type Response = String;

  async fn perform(&self, app: &App) -> KResult<Self::Response> {
    let form = types::CreateUser {
      name: self.username.as_deref(),
      email: self.email.as_opt_deref(),
      password_hash: self.password.as_deref(),
    };
    let user = app.data().create_user(&form).await?;
    todo!()
  }
}
