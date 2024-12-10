pub mod jwt;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::typed_header::TypedHeaderRejectionReason;
use axum_extra::TypedHeader;
use capwat_error::{ApiError, ApiErrorCategory};
use capwat_model::User;
use capwat_postgres::impls::UserPgImpl;
use std::fmt::Debug;

use crate::App;

type ApiResult<T> = std::result::Result<T, ApiError>;

/// This object allows to extract identity based on the token given
/// from the `Authorization` HTTP header.
///
/// There are kinds of identities that this object supports:
/// - `Guest` - They haven't provided a token yet.
/// - `User` - Regular user in a Capwat instance.
pub enum Identity {
    Guest,
    User(User),
}

impl Debug for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Guest => f.write_str("Guest"),
            Self::User(..) => f.debug_struct("User").finish_non_exhaustive(),
        }
    }
}

impl Identity {
    #[must_use]
    pub fn requires_admin(&self) -> ApiResult<&User> {
        match self {
            Self::User(user) if user.admin => Ok(user),
            _ => Err(ApiError::new(ApiErrorCategory::AccessDenied)),
        }
    }

    #[must_use]
    pub fn requires_auth(&self) -> ApiResult<&User> {
        match self {
            Self::Guest => Err(ApiError::new(ApiErrorCategory::AccessDenied)),
            Self::User(data) => Ok(data),
        }
    }
}

#[axum::async_trait]
impl FromRequestParts<App> for Identity {
    type Rejection = Response;

    // TODO: cleanup this code
    async fn from_request_parts(parts: &mut Parts, state: &App) -> Result<Self, Self::Rejection> {
        let app = App::from_request_parts(parts, state).await?;
        let header_result: Result<TypedHeader<Authorization<Bearer>>, _> =
            TypedHeader::from_request_parts(parts, state).await;

        let token = match header_result {
            Ok(header) => header.token().to_string(),
            Err(error) if matches!(error.reason(), TypedHeaderRejectionReason::Missing) => {
                return Ok(Self::Guest)
            }
            Err(..) => return Err(ApiError::new(ApiErrorCategory::AccessDenied).into_response()),
        };

        let claims = match jwt::LoginClaims::decode(&app, &token) {
            Ok(claims) => claims,
            Err(error) => return Err(error.into_api_error().into_response()),
        };

        let mut conn = match app.db_read().await {
            Ok(data) => data,
            Err(error) => return Err(error.into_api_error().into_response()),
        };

        let result = match User::find(&mut conn, claims.sub).await {
            Ok(data) => data,
            Err(error) => return Err(error.into_api_error().into_response()),
        };

        if let Some(user) = result {
            Ok(Self::User(user))
        } else {
            Err(ApiError::new(ApiErrorCategory::AccessDenied).into_response())
        }
    }
}
