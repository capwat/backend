use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use capwat_api_types::user::UserFlags;
use capwat_db::pool::PgConnection;
use capwat_error::ext::NoContextResultExt;
use capwat_error::{ApiError, ApiErrorCategory};
use capwat_model::id::UserId;
use capwat_model::user::{UserAggregates, UserView};
use capwat_model::User;
use std::ops::Deref;
use thiserror::Error;

use crate::App;

#[derive(Clone)]
pub struct SessionUser {
    pub aggregates: UserAggregates,
    pub flags: UserFlags,
    pub user: User,
}

impl Deref for SessionUser {
    // for compatibility purposes with the entire codebase
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
        let user_view = UserView::find(conn, id)
            .await
            .change_context(GetSessionUserError)?;

        if let Some(user_view) = user_view {
            Ok(Self {
                aggregates: user_view.aggregates,
                flags: user_view.flags,
                user: user_view.user,
            })
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
            .field("id", &self.id)
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
