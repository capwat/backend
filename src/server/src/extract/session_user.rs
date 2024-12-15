use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use capwat_db::pool::PgConnection;
use capwat_error::ext::NoContextResultExt;
use capwat_error::{ApiError, ApiErrorCategory};
use capwat_model::id::UserId;
use capwat_model::User;
use std::ops::Deref;
use thiserror::Error;

use crate::App;

#[derive(Clone)]
pub struct SessionUser {
    pub user: User,
}

impl SessionUser {
    #[must_use]
    pub fn into_inner(self) -> User {
        self.user
    }
}

impl Deref for SessionUser {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        &self.user
    }
}

#[derive(Debug, Error)]
#[error("could not make a session user")]
pub(crate) struct GetSessionUserError;

impl SessionUser {
    pub(crate) async fn from_db(
        conn: &mut PgConnection,
        id: UserId,
    ) -> capwat_error::Result<Self, GetSessionUserError> {
        let user = User::find(conn, id)
            .await
            .change_context(GetSessionUserError)?;

        if let Some(user) = user {
            Ok(Self { user })
        } else {
            let error =
                capwat_error::Error::new(ApiErrorCategory::AccessDenied, GetSessionUserError)
                    .attach_printable("specified user does not exists");

            Err(error)
        }
    }
}

impl std::fmt::Debug for SessionUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // for diagnostic purposes
        f.debug_struct("SessionUser")
            .field("id", &self.user.id)
            .finish_non_exhaustive()
    }
}

#[axum::async_trait]
impl FromRequestParts<App> for SessionUser {
    type Rejection = Response;

    #[tracing::instrument(skip_all, name = "extractors.session_user")]
    async fn from_request_parts(parts: &mut Parts, _app: &App) -> Result<Self, Self::Rejection> {
        match parts.extensions.get::<SessionUser>() {
            Some(identity) => Ok(identity.clone()),
            None => Err(ApiError::new(ApiErrorCategory::AccessDenied).into_response()),
        }
    }
}
