// Some structs are duplicated but they are served the purpose
// of backwards compatibility between API versions.
mod local_profile;
mod login;
mod register;

pub use self::local_profile::{LocalProfile, LocalProfileResponse};
pub use self::login::{Login, LoginResponse};
pub use self::register::{Register, RegisterResult};
