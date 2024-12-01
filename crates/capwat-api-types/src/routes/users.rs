use serde::{Deserialize, Serialize};

use crate::users::{UserClassicKeys, UserPostQuantumKeys, UserSalt};
use crate::util::Sensitive;

/// Log in as a user to Capwat.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "server", derive(bon::Builder))]
#[cfg_attr(feature = "server", builder(on(Sensitive<String>, into)))]
pub struct LoginUser {
    pub name_or_email: Sensitive<String>,
    pub access_key: Sensitive<String>,
}

/// Sign up to Capwat.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "server", derive(bon::Builder))]
#[cfg_attr(feature = "server", builder(on(Sensitive<String>, into)))]
pub struct RegisterUser {
    pub name: Sensitive<String>,
    pub email: Option<Sensitive<String>>,
    pub access_key_hash: Sensitive<String>,

    pub classic_keys: Sensitive<UserClassicKeys>,
    pub pqc_keys: Sensitive<UserPostQuantumKeys>,

    pub salt: Sensitive<UserSalt>,
}

/// A response after registration is successfully performed.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RegisterUserResponse {
    /// Whether email verification is required before logging in
    /// to the Capwat instance.
    pub verify_email: bool,
}
