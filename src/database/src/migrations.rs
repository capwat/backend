use capwat_error::ext::ResultExt;
use capwat_error::Result;
use diesel_async_migrations::EmbeddedMigrations;
use std::ops::DerefMut;
#[cfg(not(test))]
use tokio::time::Instant;
#[cfg(not(test))]
use tracing::info;

use crate::error::MigrationError;
use crate::pool::PgConnection;

#[tracing::instrument(skip_all, name = "migrations.run_pending")]
pub async fn run_pending<'a>(
    conn: &mut PgConnection<'a>,
    migrations: &EmbeddedMigrations,
) -> Result<(), MigrationError> {
    #[cfg(not(test))]
    let now = Instant::now();

    #[cfg(not(test))]
    info!("Performing database migrations... (this may take a while)");

    migrations
        .run_pending_migrations(conn.deref_mut())
        .await
        .change_context(MigrationError)?;

    #[cfg(not(test))]
    {
        let elapsed = now.elapsed();
        info!("Successfully performed database migrations! took {elapsed:.2?}");
    }

    Ok(())
}
