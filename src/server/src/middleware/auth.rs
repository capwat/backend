use axum::extract::{FromRequestParts, Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum_extra::either::Either;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use capwat_db::pool::PgConnection;
use capwat_error::ext::ResultExt;
use capwat_error::Result;
use capwat_model::id::UserId;

use crate::auth::jwt::LoginClaims;
use crate::extract::{LocalInstanceSettings, SessionUser};
use crate::services::util::check_email_status;
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
    request: Request,
    next: Next,
) -> Response {
    let request = if let Some(header) = metadata.auth_header {
        match process_user_token(&app, request, header.token()).await {
            Ok(Either::E1(request)) => request,
            Ok(Either::E2(response)) => return response,
            Err(error) => return error.into_api_error().into_response(),
        }
    } else {
        request
    };
    next.run(request).await
}

async fn process_user_token(
    app: &App,
    request: Request,
    token: &str,
) -> Result<Either<Request, Response>> {
    let mut conn = app.db_read().await?;
    let user = get_user_from_token(&mut conn, app, token).await?;

    drop(conn);

    let (mut parts, body) = request.into_parts();
    let settings = match LocalInstanceSettings::from_request_parts(&mut parts, app).await {
        Ok(settings) => settings,
        Err(error) => return Ok(Either::E2(error.into_response())),
    };

    match check_email_status(&user, &settings) {
        Ok(..) => {}
        Err(error) => return Ok(Either::E2(error.into_response())),
    };

    let mut request = Request::from_parts(parts, body);
    request.extensions_mut().insert(user);

    Ok(Either::E1(request))
}

async fn get_user_from_token(
    conn: &mut PgConnection<'_>,
    app: &App,
    token: &str,
) -> Result<SessionUser> {
    let claims = LoginClaims::decode(&app, token)?;
    SessionUser::from_db(conn, UserId(claims.sub))
        .await
        .erase_context()
}
