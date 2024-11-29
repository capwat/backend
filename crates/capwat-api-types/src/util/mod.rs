#[cfg(feature = "server")]
mod sensitive;

#[cfg(feature = "server")]
pub use self::sensitive::Sensitive;

#[cfg(not(feature = "server"))]
pub type Sensitive<T> = T;
