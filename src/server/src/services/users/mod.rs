// Some structs are duplicated but they're served the purpose to
// implement backwards compatibility between API versions.
mod login;
mod register;

pub mod profile;

pub use self::login::{Login, LoginResponse};
pub use self::register::{Register, RegisterResult};
