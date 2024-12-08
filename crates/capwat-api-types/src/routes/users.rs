use serde::{Deserialize, Serialize};

use crate::encrypt::ClassicKey;
#[cfg(feature = "experimental")]
use crate::user::UserPostQuantumKeys;
use crate::user::{UserClassicKeys, UserSalt};

use crate::util::{EncodedBase64, Sensitive};

/// A response after `GET /users/@me` or `GET /users/:id` has successfully performed.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LocalUserProfile {
    pub id: i64,
    pub name: String,
    pub display_name: Option<String>,
    pub classic_public_key: ClassicKey,
}

/// Log in as a user to Capwat.
///
/// **ROUTE**: `POST /users/login`
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "server", derive(bon::Builder))]
pub struct LoginUser {
    #[cfg_attr(feature = "server", builder(into))]
    pub name_or_email: Sensitive<String>,

    /// This field is optional as we need to get the user's
    /// salt if needed.
    #[cfg_attr(feature = "server", builder(into))]
    pub access_key_hash: Option<Sensitive<EncodedBase64>>,
}

/// A response after [logging in as a user] has successfully performed.
///
/// [logging in as a user]: LoginUser
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LoginUserResponse {
    pub name: String,
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    pub classic_keys: UserClassicKeys,
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
    pub name: Sensitive<String>,
    pub email: Option<Sensitive<String>>,

    #[cfg_attr(feature = "server", builder(into))]
    pub salt: Sensitive<UserSalt>,

    #[cfg_attr(feature = "server", builder(into))]
    pub access_key_hash: Sensitive<EncodedBase64>,

    #[cfg_attr(feature = "server", builder(into))]
    pub symmetric_key: Sensitive<EncodedBase64>,

    #[cfg_attr(feature = "server", builder(into))]
    pub classic_keys: Sensitive<UserClassicKeys>,

    #[cfg(feature = "experimental")]
    #[cfg_attr(feature = "server", builder(into))]
    pub post_quantum_keys: Sensitive<UserPostQuantumKeys>,
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
