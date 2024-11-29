use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::Response;

use super::LocalInstanceSettings;
use crate::App;

pub struct RequiresCaptcha;

#[axum::async_trait]
impl FromRequestParts<App> for RequiresCaptcha {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &App) -> Result<Self, Self::Rejection> {
        let _app = App::from_request_parts(parts, state).await?;
        let settings = LocalInstanceSettings::from_request_parts(parts, state).await?;
        if !settings.require_captcha {
            return Ok(Self);
        }

        todo!()
    }
}
