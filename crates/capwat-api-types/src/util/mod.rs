mod base64;
pub use self::base64::EncodedBase64;

#[cfg(feature = "server")]
mod sensitive;

#[cfg(feature = "server")]
pub use self::sensitive::Sensitive;

#[cfg(not(feature = "server"))]
pub type Sensitive<T> = T;
