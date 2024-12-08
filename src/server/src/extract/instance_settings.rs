use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use capwat_model::instance::InstanceSettings;
use capwat_postgres::impls::InstanceSettingsPgImpl;
use std::ops::Deref;

use crate::App;

/// Gets the [local instance settings] directly upon request.
///
/// [local instance settings]: InstanceSettings
pub struct LocalInstanceSettings(pub InstanceSettings);

impl Deref for LocalInstanceSettings {
    type Target = InstanceSettings;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[axum::async_trait]
impl FromRequestParts<App> for LocalInstanceSettings {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &App) -> Result<Self, Self::Rejection> {
        let app = App::from_request_parts(parts, state).await?;

        let mut conn = app
            .db_read()
            .await
            .map_err(|e| e.into_api_error().into_response())?;

        let settings = InstanceSettings::get_local(&mut conn)
            .await
            .map_err(|e| e.into_api_error().into_response())?;

        Ok(Self(settings))
    }
}
