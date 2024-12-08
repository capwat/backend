use std::str::FromStr;

use axum::response::{IntoResponse, Response};
use capwat_api_types::encrypt::ClassicKey;
use capwat_api_types::routes::users::LocalUserProfile;
use capwat_error::ext::ResultExt;
use capwat_error::ApiError;
use capwat_model::user::UserKeys;
use capwat_postgres::impls::UserKeysPgImpl;

use crate::auth::Identity;
use crate::extract::Json;
use crate::App;

#[tracing::instrument(skip(app), name = "v1.users.local_profile")]
pub async fn local_profile(app: App, identity: Identity) -> Result<Response, ApiError> {
    let user = identity.requires_auth()?;

    let mut conn = app.db_read().await?;
    let user_keys = UserKeys::get_current(&mut conn, user.id).await?;
    let response = Json(LocalUserProfile {
        id: user.id.0,
        name: user.name.clone(),
        display_name: user.display_name.clone(),
        classic_public_key: ClassicKey::from_str(&user_keys.public_key)
            .attach_printable("invalid user classic public key")?,
    });

    Ok(response.into_response())
}
