#![allow(async_fn_in_trait)]

pub mod error;
pub mod ext;
pub mod impls;
pub mod migrations;
pub mod pool;
pub mod transaction;

pub use self::pool::PgPool;

pub(crate) mod internal;

mod test;

use self::error::ConnectError;
use capwat_error::ext::{NoContextResultExt, ResultExt};
use capwat_error::{ApiErrorCategory, Error, Result};
use diesel_async::{AsyncConnection, AsyncPgConnection};
use url::Url;

/// Installs [`capwat_error`] middleware for types [`diesel::result::Error`]
#[allow(deprecated)]
pub fn install_error_middleware() {
    use capwat_error::middleware::impls::Report;
    use diesel::result::{DatabaseErrorKind, Error as DieselError};

    Error::install_middleware::<DieselError>(|error, location, category| {
        match &error {
            DieselError::BrokenTransactionManager
            | DieselError::DatabaseError(DatabaseErrorKind::ClosedConnection, ..) => {
                *category = ApiErrorCategory::Outage;
            }
            DieselError::DatabaseError(DatabaseErrorKind::ReadOnlyTransaction, ..) => {
                *category = ApiErrorCategory::ReadonlyMode;
            }
            _ => {}
        };
        Report::new_without_location(error)
            .attach_location(Some(location))
            .erase_context()
    });
}

/// Establishes a connection straight from `DATABASE_URL` or `CAPWAT_CLI_DB_URL`
/// environment variable. This is useful for administrative-related tasks.
pub async fn connection_from_env() -> Result<AsyncPgConnection, ConnectError> {
    fn should_connect_with_tls(url: &str) -> Result<bool, ConnectError> {
        let required_from_env =
            capwat_utils::env::var_opt_parsed::<bool>(capwat_config::vars::ADMIN_CLI_DB_USE_TLS)
                .change_context(ConnectError)?
                .unwrap_or(false);

        let url = Url::parse(url)
            .change_context(ConnectError)
            .attach_printable("invalid Postgres connection URL")?;

        let required_from_url = url.query_pairs().any(|(key, _)| {
            matches!(
                key.to_lowercase().as_str(),
                "require" | "verify-ca" | "verify-full"
            )
        });

        Ok(required_from_url || required_from_env)
    }

    let url = capwat_utils::env::var_opt(capwat_config::vars::ADMIN_CLI_DB_URL2)
        .transpose()
        .unwrap_or_else(|| capwat_utils::env::var(capwat_config::vars::ADMIN_CLI_DB_URL))
        .change_context(ConnectError)?;

    // we can check whether we need to connect to the database
    // with TLS encryption tunnel through `sslmode` in its URL
    // queries or `CAPWAT_CLI_DB_USE_TLS`.
    let should_use_tls = should_connect_with_tls(&url)?;
    let connection = if should_use_tls {
        self::internal::establish_connection_with_tls(&url)
            .await
            .change_context(ConnectError)
            .attach_printable("could not establish Postgres connection with TLS")
    } else {
        AsyncPgConnection::establish(&url)
            .await
            .change_context(ConnectError)
            .attach_printable("could not establish Postgres connection")
    }?;

    Ok(connection)
}
