use capwat_error::{ext::ResultExt, ApiErrorCategory, Result};
use diesel::backend::Backend;
use diesel::pg::Pg;
use diesel::query_builder::{QueryBuilder, QueryFragment};
use diesel_async::{AnsiTransactionManager, AsyncPgConnection, TransactionManager};
use std::ops::DerefMut;
use tracing::trace;

use crate::{
    error::{BeginTransactError, CommitTransactError, RollbackTransactError},
    pool::PgConnection,
};

mod builder;
pub use self::builder::TransactionBuilder;

pub struct Transaction<'a> {
    connection: Option<PgConnection<'a>>,
    is_testing_connection: bool,
}

impl<'a> Transaction<'a> {
    #[tracing::instrument(name = "db.transaction.begin")]
    pub(crate) async fn new(builder: TransactionBuilder<'a>) -> Result<Self, BeginTransactError> {
        let mut query_builder = <Pg as Backend>::QueryBuilder::default();
        builder
            .to_sql(&mut query_builder, &Pg)
            .change_context(BeginTransactError)
            .attach_printable("could not build transaction SQL")?;

        let sql = query_builder.finish();

        let mut conn = builder.connection;
        AnsiTransactionManager::begin_transaction_sql(&mut *conn, &sql)
            .await
            .change_context(BeginTransactError)
            .category(ApiErrorCategory::Outage)
            .attach_printable("could not begin PostgreSQL transaction")?;

        Ok(Self {
            connection: Some(conn),
            is_testing_connection: builder.is_testing,
        })
    }

    #[tracing::instrument(skip(self), name = "db.transaction.commit")]
    pub async fn commit(mut self) -> Result<(), CommitTransactError> {
        trace!("commiting transaction...");
        let mut conn = self.connection.take().expect("self.connection is dropped");
        AnsiTransactionManager::commit_transaction(conn.deref_mut())
            .await
            .change_context(CommitTransactError)?;

        trace!("commiting done");
        Ok(())
    }

    #[tracing::instrument(skip(self), name = "db.transaction.rollback")]
    pub async fn rollback(mut self) -> Result<(), RollbackTransactError> {
        let mut conn = self.connection.take().expect("self.connection is dropped");
        AnsiTransactionManager::rollback_transaction(conn.deref_mut())
            .await
            .change_context(RollbackTransactError)?;

        Ok(())
    }
}

impl<'a> std::ops::Deref for Transaction<'a> {
    type Target = PgConnection<'a>;

    fn deref(&self) -> &Self::Target {
        self.connection
            .as_ref()
            .expect("self.connection is dropped")
    }
}

impl std::ops::DerefMut for Transaction<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.connection
            .as_mut()
            .expect("self.connection is dropped")
    }
}

async fn try_rollback<T: DerefMut<Target = AsyncPgConnection>>(mut conn: T) {
    let conn = conn.deref_mut();
    if let Err(error) = AnsiTransactionManager::rollback_transaction(conn)
        .await
        .change_context(RollbackTransactError)
    {
        tracing::error!(%error, "Failed to rollback transaction");
    }
}

// This method does not guarantee that a connection can be successfully
// rollback the transaction! :)
impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        // We don't need to do anything if it is already committed
        // or rollbacked by a method/function
        let Some(conn) = self.connection.take() else {
            return;
        };

        // Pooled connection will go after
        match conn {
            // I mean it's impossible to have some kind of pooled
            // connection in testing suite but who cares anyway :)
            PgConnection::Pooled(conn) => {
                tokio::spawn(try_rollback(conn));
            }
            PgConnection::Raw(conn) => tokio::task::block_in_place(|| {
                futures::executor::block_on(try_rollback(conn));
            }),
        }
    }
}
