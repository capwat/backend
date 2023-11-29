use async_trait::async_trait;
use capwat_kernel::error::Result;
use capwat_types::forms;

use crate::{App, Perform};

#[async_trait]
impl Perform for forms::Login {
  type Response = forms::LoginResponse;

  async fn perform(&self, app: App) -> Result<Self::Response> {
    let Some(user) =
      app.data.find_user_by_login(&self.username_or_email).await?
    else {
      todo!()
    };

    todo!()
  }
}
