use actix_web::{http::header, web, FromRequest};
use futures::future::{ready, LocalBoxFuture};
use thiserror::Error;

use crate::{schema::User, App};

use super::{Error, Jwt};

#[derive(Debug)]
pub enum Actor {
  Anonymous,
  User(User),
}

impl Actor {
  pub fn get_user(self) -> Result<User, Error> {
    #[derive(Debug, Error)]
    #[error("Attempt to access user-only route")]
    struct Unauthorized;
    match self {
      Self::User(n) => Ok(n),
      Self::Anonymous => Err(Error::from_context(
        crate::types::Error::Unauthorized,
        Unauthorized,
      )),
    }
  }
}

impl FromRequest for Actor {
  type Error = Error;
  type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

  fn from_request(
    req: &actix_web::HttpRequest,
    _payload: &mut actix_web::dev::Payload,
  ) -> Self::Future {
    let token = req
      .headers()
      .get(header::AUTHORIZATION)
      .and_then(|v| v.to_str().ok())
      .and_then(|v| v.strip_prefix("Bearer "));

    if let Some(token) = token {
      let Some(app) = req.app_data::<web::Data<App>>() else {
        #[derive(Debug, Error)]
        #[error("The web app has no available configuration")]
        struct NoConfig;
        return Box::pin(ready(Err(Error::from_context(
          crate::types::Error::Internal,
          NoConfig,
        ))));
      };

      let app = app.clone();
      let jwt = Jwt::decode(token, app.as_ref());
      Box::pin(async move {
        let mut conn = app.db_read_prefer_primary().await?;
        if let Some(user) = User::by_id(&mut *conn, jwt.user_id).await? {
          Ok(Actor::User(user))
        } else {
          Ok(Actor::Anonymous)
        }
      })
    } else {
      Box::pin(ready(Ok(Actor::Anonymous)))
    }
  }
}
