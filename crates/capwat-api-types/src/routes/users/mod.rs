use serde::{Deserialize, Serialize};

use crate::user::UserSalt;
use crate::util::{EncodedBase64, Timestamp};

mod current_user;
pub use self::current_user::*;

/// Log in as a user to Capwat.
///
/// **ROUTE**: `POST /users/login`
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "server", derive(bon::Builder))]
pub struct LoginUser {
    #[cfg_attr(feature = "server", builder(into))]
    pub name_or_email: String,

    /// This field is optional as we need to get the user's
    /// salt if needed.
    #[cfg_attr(feature = "server", builder(into))]
    pub access_key_hash: Option<EncodedBase64>,
}

/// A response after [logging in as a user] has successfully performed.
///
/// [logging in as a user]: LoginUser
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LoginUserResponse {
    pub id: i64,
    pub name: String,
    pub joined_at: Timestamp,
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    pub encrypted_symmetric_key: String,
    pub token: String,
}

/// Sign up to Capwat.
///
/// **ROUTE**: `POST /users/register`
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "server", derive(bon::Builder))]
#[cfg_attr(feature = "server", builder(on(Sensitive<String>, into)))]
pub struct RegisterUser {
    pub name: String,
    pub email: Option<String>,

    #[cfg_attr(feature = "server", builder(into))]
    pub salt: UserSalt,

    #[cfg_attr(feature = "server", builder(into))]
    pub access_key_hash: EncodedBase64,

    #[cfg_attr(feature = "server", builder(into))]
    pub symmetric_key: EncodedBase64,
}

/// A response after [registration] has successfully performed.
///
/// [registration]: RegisterUser
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RegisterUserResponse {
    /// Whether email verification is required before logging in
    /// to the Capwat instance.
    pub verify_email: bool,
}

crate::should_impl_primitive_traits!(LoginUser);
crate::should_impl_primitive_traits!(LoginUserResponse);
crate::should_impl_primitive_traits!(RegisterUser);
crate::should_impl_primitive_traits!(RegisterUserResponse);
