// TODO: Implement snowflake for Capwat

pub mod error;
pub mod post;
pub mod routes;
pub mod user;
pub mod util;

pub use self::error::{Error, ErrorCategory};

#[allow(unused)]
macro_rules! should_impl_primitive_traits {
    ($ty:ty) => {
        #[cfg(test)]
        ::static_assertions::assert_impl_all!($ty: std::fmt::Debug,
            Clone,
            PartialEq,
            Eq,
            ::serde::de::DeserializeOwned,
            ::serde::Serialize
        );
    };
}
pub(crate) use should_impl_primitive_traits;
