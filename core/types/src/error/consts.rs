pub const INTERNAL_CODE: u32 = 1;
pub const READONLY_MODE_CODE: u32 = 2;
pub const NOT_AUTHENTICATED_CODE: u32 = 3;
pub const INVALID_FORM_BODY_CODE: u32 = 4;
pub const LOGIN_USER_CODE: u32 = 5;

pub const INTERNAL_MSG: &str =
  "Internal server occurred. Please try again later.";

pub const READONLY_MODE_MSG: &str =
  "This service is currently in read only mode. Please try again later.";

pub const NOT_AUTHENTICATED_MSG: &str = "Not authenticated";
pub const INVALID_FORM_BODY_MSG: &str = "Invalid form body";

pub mod login_user {
  pub const INVALID_CREDIENTIALS_MSG: &str = "Failed to login session";
  pub const BANNED_MSG: &str = "Your account is terminated";

  pub const INVALID_CREDIENTIALS_CODE: u32 = 1;
  pub const BANNED_CODE: u32 = 2;
}
