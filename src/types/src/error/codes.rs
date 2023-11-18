pub const SERVER: u32 = 1;
pub mod server {
  pub const INTERNAL: u32 = 1;
  pub const READONLY_MODE: u32 = 2;
  // pub const OUTAGE: u32 = 3;
}

pub const INVALID_REQUEST: u32 = 2;
pub mod invalid_request {
  pub const UNSUPPORTED_API_VERSION: u32 = 1;
  pub const INVALID_FORM_BODY: u32 = 2;
  pub const CONTENT_TYPE: u32 = 3;
}

pub const LOGIN_USER: u32 = 3;
pub mod login_user {
  pub const INVALID_CREDENTIALS: u32 = 1;
}

pub const REGISTER_USER: u32 = 4;
pub mod register_user {
  pub const CLOSED: u32 = 1;
  pub const EMAIL_REQUIRED: u32 = 2;
  pub const EMAIL_EXISTS: u32 = 3;
  pub const USER_EXISTS: u32 = 4;
}
