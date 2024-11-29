use serde::{Deserialize, Serialize};

use crate::util::Sensitive;

/// Log in as a user to Capwat.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "server", derive(bon::Builder))]
#[cfg_attr(feature = "server", builder(on(Sensitive<String>, into)))]
pub struct LoginUser {
    pub name_or_email: Sensitive<String>,
    pub password: Sensitive<String>,
}

/// Sign up to Capwat.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "server", derive(bon::Builder))]
#[cfg_attr(feature = "server", builder(on(Sensitive<String>, into)))]
pub struct RegisterUser {
    pub name: Sensitive<String>,
    pub email: Option<Sensitive<String>>,

    // TODO: Let the user generate their own password hashes
    pub password: Sensitive<String>,
    pub password_verify: Sensitive<String>,
}

/// A response after registration is successfully performed.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RegisterUserResponse {
    /// Whether email verification is required before logging in
    /// to the Capwat instance.
    pub verify_email: bool,
}
