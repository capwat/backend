use capwat_kernel::error::ext::{ErrorExt, ResultExt};
use capwat_kernel::Result;
use diesel::pg::Pg;
use diesel::query_builder::{QueryBuilder, QueryFragment};
use diesel_async::{AnsiTransactionManager, TransactionManager};
use std::ops::{Deref, DerefMut};

use self::error::*;
use crate::{internal, Connection};

mod builder;
mod error;

pub use builder::TransactionBuilder;

pub struct Transaction<'a> {
    connection: Option<Connection<'a>>,
    terminated: bool,
}

impl<'a> Transaction<'a> {
    #[tracing::instrument]
    pub(crate) async fn new(builder: TransactionBuilder<'a>) -> Result<Self> {
        let mut query_builder =
            <Pg as diesel::backend::Backend>::QueryBuilder::default();

        builder
            .to_sql(&mut query_builder, &Pg)
            .into_error()
            .change_context(BeginFailed)?;

        let sql = query_builder.finish();

        let mut conn = builder.connection;
        AnsiTransactionManager::begin_transaction_sql(&mut *conn, &sql)
            .await
            .into_error()
            .change_context(BeginFailed)?;

        Ok(Self { connection: Some(conn), terminated: false })
    }

    #[tracing::instrument(skip(self))]
    pub async fn commit(mut self) -> Result<()> {
        let conn =
            self.connection.as_mut().expect("self.connection is dropped");

        AnsiTransactionManager::commit_transaction(conn.deref_mut())
            .await
            .into_error()
            .change_context(CommitFailed)?;

        self.terminated = true;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn rollback(mut self) -> Result<()> {
        let conn =
            self.connection.as_mut().expect("self.connection is dropped");

        AnsiTransactionManager::rollback_transaction(conn.deref_mut())
            .await
            .into_error()
            .change_context(RollbackFailed)?;

        self.terminated = true;
        Ok(())
    }
}

impl<'a> Deref for Transaction<'a> {
    type Target = internal::PgConnection;

    fn deref(&self) -> &Self::Target {
        self.connection.as_ref().expect("self.connection is dropped")
    }
}

impl<'a> DerefMut for Transaction<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.connection.as_mut().expect("self.connection is dropped")
    }
}

// This method does not guarantee that a connection can be successfully
// rollback the transaction! :)
impl<'a> Drop for Transaction<'a> {
    fn drop(&mut self) {
        // We don't need to do anything if it is already committed
        // or rollbacked by a method/function
        if self.terminated {
            return;
        }

        let Some(conn) = self.connection.take() else { return };

        // Pooled connection will go after
        let conn = match conn {
            Connection::Pool(n) => n,
            Connection::Test(mut conn) => {
                // It's just testing environment.
                futures::executor::block_on(async move {
                    if let Err(error) =
                        AnsiTransactionManager::rollback_transaction(&mut *conn)
                            .await
                            .into_error()
                            .change_context(CommitFailed)
                    {
                        tracing::error!(
                            ?error,
                            "Failed to rollback transaction"
                        );
                    }
                });
                return;
            },
        };

        // WTF? HOW?
        tokio::spawn(async move {
            let mut conn = conn;
            if let Err(err) =
                AnsiTransactionManager::rollback_transaction(&mut *conn).await
            {
                tracing::error!(?err, "Failed to rollback transaction");
            }
        });
    }
}
