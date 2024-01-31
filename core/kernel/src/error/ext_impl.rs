#[cfg(feature = "diesel")]
impl super::ext::IntoError for diesel::ConnectionError {
    fn into_error(self) -> crate::Error {
        match self {
            diesel::ConnectionError::CouldntSetupConfiguration(n) => {
                n.into_error()
            },
            _ => crate::Error::internal(self),
        }
    }
}

// Rust does not allow us to implement traits outside of their
// crate with objects from different crates.
#[cfg(feature = "diesel")]
impl super::ext::IntoError for diesel::result::Error {
    fn into_error(self) -> crate::Error {
        use crate::error::{Category, Error};
        match self {
            diesel::result::Error::DatabaseError(_, ref info)
                if info.message().ends_with("read-only transaction") =>
            {
                Error::from_context(Category::ReadonlyMode, self)
            },
            diesel::result::Error::NotFound => {
                Error::from_context(Category::NotFound, self)
            },
            _ => crate::Error::internal(self),
        }
    }
}
