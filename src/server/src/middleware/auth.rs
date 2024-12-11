use axum::extract::{FromRequestParts, Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use capwat_error::ext::ResultExt;
use capwat_error::Result;
use capwat_model::id::UserId;

use crate::auth::jwt::LoginClaims;
use crate::extract::SessionUser;
use crate::App;

#[doc(hidden)]
#[derive(FromRequestParts)]
pub struct Metadata {
    auth_header: Option<TypedHeader<Authorization<Bearer>>>,
}

#[tracing::instrument(skip_all, name = "middleware.auth")]
pub async fn catch_token(
    metadata: Metadata,
    app: State<App>,
    mut request: Request,
    next: Next,
) -> Response {
    if let Some(header) = metadata.auth_header {
        let user = match get_user_from_token(&app, header.token()).await {
            Ok(data) => data,
            Err(error) => return error.into_api_error().into_response(),
        };
        request.extensions_mut().insert(user);
    };
    next.run(request).await
}

async fn get_user_from_token(app: &App, token: &str) -> Result<SessionUser> {
    let claims = LoginClaims::decode(&app, token)?;

    let mut conn = app.db_read().await?;
    SessionUser::from_db(&mut conn, UserId(claims.sub))
        .await
        .erase_context()
}
