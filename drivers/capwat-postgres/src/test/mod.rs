use std::sync::{atomic::AtomicBool, LazyLock};
use tracing::warn;
use url::Url;

mod pool;
mod schema;

pub use self::pool::TestPool;

static DO_CLEANUP: AtomicBool = AtomicBool::new(true);
static DATABASE_URL: LazyLock<Url> = LazyLock::new(|| {
    let result = capwat_utils::env::var_opt_parsed::<Url>("CAPWAT_DB_PRIMARY_URL")
        .transpose()
        .or_else(|| {
            capwat_utils::env::var_opt_parsed::<Url>(capwat_config::vars::ADMIN_CLI_DB_URL)
                .transpose()
        })
        .unwrap_or_else(|| {
            capwat_utils::env::var_parsed::<Url>(capwat_config::vars::ADMIN_CLI_DB_URL2)
        });

    match result {
        Ok(that) => that,
        Err(error) => {
            eprintln!("could not get database url for testing: {error:#}");
            std::process::exit(1);
        }
    }
});

static DATABASE_USE_TLS: LazyLock<bool> = LazyLock::new(|| {
    let result =
        capwat_utils::env::var_opt_parsed::<bool>(capwat_config::vars::ADMIN_CLI_DB_USE_TLS)
            .transpose()
            .unwrap_or_else(|| {
                capwat_utils::env::var_parsed::<bool>(capwat_config::vars::DB_ENFORCE_TLS)
            });

    match result {
        Ok(that) => that,
        Err(error) => {
            warn!(%error, "Could not determine whether it needs to connect to the database with TLS encryption for testing. disabling TLS encryption for testing database");
            false
        }
    }
});
