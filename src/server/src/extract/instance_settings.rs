use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use capwat_model::instance::InstanceSettings;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;
use tracing::trace;

use crate::App;

/// Gets the [local instance settings] directly upon request.
///
/// [local instance settings]: InstanceSettings
pub struct LocalInstanceSettings(pub Arc<InstanceSettings>);

impl LocalInstanceSettings {
    #[must_use]
    pub fn new(value: InstanceSettings) -> Self {
        Self(Arc::new(value))
    }
}

impl Debug for LocalInstanceSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        InstanceSettings::fmt(&self.0, f)
    }
}

impl Deref for LocalInstanceSettings {
    type Target = InstanceSettings;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[axum::async_trait]
impl FromRequestParts<App> for LocalInstanceSettings {
    type Rejection = Response;

    #[tracing::instrument(skip_all, name = "extractors.instance.settings")]
    async fn from_request_parts(parts: &mut Parts, app: &App) -> Result<Self, Self::Rejection> {
        // We'll going to cache this data because some extractors like
        // `Identity` in auth module needs that.
        if let Some(ptr) = parts.extensions.get::<Arc<InstanceSettings>>() {
            trace!("cache hit! using the local copy from fetched instance settings");
            Ok(Self(ptr.clone()))
        } else {
            trace!("cache miss! fetching from DB");
            let mut conn = app
                .db_read()
                .await
                .map_err(|e| e.into_api_error().into_response())?;

            let settings = InstanceSettings::get_local(&mut conn)
                .await
                .map_err(|e| e.into_api_error().into_response())?;

            let ptr = Arc::new(settings);
            parts.extensions.insert(ptr.clone());

            Ok(Self(ptr))
        }
    }
}
