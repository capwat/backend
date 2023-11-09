use sensitive::Sensitive;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct Request {
    #[validate(with = "crate::validation::is_valid_username")]
    pub username: Sensitive<String>,
    #[validate(with = "crate::validation::is_valid_email", optional)]
    pub email: Option<Sensitive<String>>,
    #[validate(length(min = 12, max = 128))]
    pub password: Sensitive<String>,
    #[validate(length(min = 12, max = 128))]
    pub confirm_password: Sensitive<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Response {
    // For e-mails only and verification is required depending
    // on the feelings of the Whim instance maintainer.
    pub verification_required: bool,
}
