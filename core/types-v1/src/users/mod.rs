use capwat_types_common::Sensitive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Login {
    pub username_or_email: Sensitive<String>,
    pub password: Sensitive<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Register {
    pub username: Sensitive<String>,
    pub email: Sensitive<Option<String>>,
    pub password: Sensitive<String>,
    pub confirm_password: Sensitive<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PasswordReset {
    pub email: Sensitive<String>,
}
