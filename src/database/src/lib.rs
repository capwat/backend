use capwat_error::{ApiErrorCategory, Error};

pub mod error;
pub mod pool;

// Our custom implementation of testing similar to sqlx::testing
pub mod testing;

pub use self::pool::{PgConnection, PgPool, PgPooledConnection, PgTransaction};

/// Installs [`capwat_error`] middleware for types [`sqlx::Error`]
#[allow(deprecated)]
pub fn install_error_middleware() {
    use capwat_error::middleware::impls::Report;
    use sqlx::Error as SqlxError;
    use std::sync::OnceLock;

    // it happens when i tried to test something...
    static IS_INSTALLED: OnceLock<()> = OnceLock::new();
    if IS_INSTALLED.get().is_some() {
        return;
    }
    let _ = IS_INSTALLED.set(());

    Error::install_middleware::<SqlxError>(|error, location, category| {
        match &error {
            SqlxError::RowNotFound => {
                *category = ApiErrorCategory::NotFound;
            }
            SqlxError::Database(metadata) => {
                if metadata.message().contains("read-only transaction") {
                    *category = ApiErrorCategory::ReadonlyMode;
                }
            }
            SqlxError::PoolClosed | SqlxError::PoolTimedOut => {
                *category = ApiErrorCategory::Outage;
            }
            _ => {}
        }
        Report::new_without_location(error)
            .attach_location(Some(location))
            .erase_context()
    });
}
