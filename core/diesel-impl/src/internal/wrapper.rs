use diesel::connection::SimpleConnection;
use futures::{Stream, StreamExt};
use std::pin::Pin;

pub struct AsyncConnectionWrapper<C> {
    inner: C,
}

impl<C> From<C> for AsyncConnectionWrapper<C>
where
    C: diesel_async::AsyncConnection,
{
    fn from(inner: C) -> Self {
        Self { inner }
    }
}

impl<C> diesel::connection::SimpleConnection for AsyncConnectionWrapper<C>
where
    C: diesel_async::SimpleAsyncConnection,
{
    fn batch_execute(&mut self, query: &str) -> diesel::QueryResult<()> {
        let f = self.inner.batch_execute(query);
        futures::executor::block_on(f)
    }
}

impl<C> diesel::connection::ConnectionSealed for AsyncConnectionWrapper<C> {}

impl<C> diesel::connection::Connection for AsyncConnectionWrapper<C>
where
    C: diesel_async::AsyncConnection,
{
    type Backend = C::Backend;

    type TransactionManager = AsyncConnectionWrapperTransactionManagerWrapper;

    fn establish(database_url: &str) -> diesel::ConnectionResult<Self> {
        let f = C::establish(database_url);
        let inner = futures::executor::block_on(f)?;
        Ok(Self { inner })
    }

    fn execute_returning_count<T>(
        &mut self,
        source: &T,
    ) -> diesel::QueryResult<usize>
    where
        T: diesel::query_builder::QueryFragment<Self::Backend>
            + diesel::query_builder::QueryId,
    {
        let f = self.inner.execute_returning_count(source);
        futures::executor::block_on(f)
    }

    fn transaction_state(
        &mut self,
    ) -> &mut <Self::TransactionManager as diesel::connection::TransactionManager<Self>>::TransactionStateData{
        self.inner.transaction_state()
    }
}

impl<C> diesel::connection::LoadConnection for AsyncConnectionWrapper<C>
where
    C: diesel_async::AsyncConnection,
{
    type Cursor<'conn, 'query> = AsyncCursorWrapper<C::Stream<'conn, 'query>>
    where
        Self: 'conn;

    type Row<'conn, 'query> = C::Row<'conn, 'query>
    where
        Self: 'conn;

    fn load<'conn, 'query, T>(
        &'conn mut self,
        source: T,
    ) -> diesel::QueryResult<Self::Cursor<'conn, 'query>>
    where
        T: diesel::query_builder::Query
            + diesel::query_builder::QueryFragment<Self::Backend>
            + diesel::query_builder::QueryId
            + 'query,
        Self::Backend: diesel::expression::QueryMetadata<T::SqlType>,
    {
        let f = self.inner.load(source);
        let stream = futures::executor::block_on(f)?;

        Ok(AsyncCursorWrapper { stream: Box::pin(stream) })
    }
}

pub struct AsyncCursorWrapper<S> {
    stream: Pin<Box<S>>,
}

impl<S> Iterator for AsyncCursorWrapper<S>
where
    S: Stream,
{
    type Item = S::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let f = self.stream.next();
        futures::executor::block_on(f)
    }
}

pub struct AsyncConnectionWrapperTransactionManagerWrapper;

impl<C> diesel::connection::TransactionManager<AsyncConnectionWrapper<C>>
    for AsyncConnectionWrapperTransactionManagerWrapper
where
    C: diesel_async::AsyncConnection,
{
    type TransactionStateData =
            <C::TransactionManager as diesel_async::TransactionManager<C>>::TransactionStateData;

    fn begin_transaction(
        conn: &mut AsyncConnectionWrapper<C>,
    ) -> diesel::QueryResult<()> {
        let f = <C::TransactionManager as diesel_async::TransactionManager<
            _,
        >>::begin_transaction(&mut conn.inner);
        futures::executor::block_on(f)
    }

    fn rollback_transaction(
        conn: &mut AsyncConnectionWrapper<C>,
    ) -> diesel::QueryResult<()> {
        let f = <C::TransactionManager as diesel_async::TransactionManager<
            _,
        >>::rollback_transaction(&mut conn.inner);
        futures::executor::block_on(f)
    }

    fn commit_transaction(
        conn: &mut AsyncConnectionWrapper<C>,
    ) -> diesel::QueryResult<()> {
        let f = <C::TransactionManager as diesel_async::TransactionManager<
            _,
        >>::commit_transaction(&mut conn.inner);
        futures::executor::block_on(f)
    }

    fn transaction_manager_status_mut(
        conn: &mut AsyncConnectionWrapper<C>,
    ) -> &mut diesel::connection::TransactionManagerStatus {
        <C::TransactionManager as diesel_async::TransactionManager<_>>::transaction_manager_status_mut(
                &mut conn.inner,
            )
    }

    fn is_broken_transaction_manager(
        conn: &mut AsyncConnectionWrapper<C>,
    ) -> bool {
        <C::TransactionManager as diesel_async::TransactionManager<_>>::is_broken_transaction_manager(
                &mut conn.inner,
            )
    }
}

impl<C> diesel::migration::MigrationConnection for AsyncConnectionWrapper<C>
where
    Self: diesel::Connection,
{
    fn setup(&mut self) -> diesel::QueryResult<usize> {
        self.batch_execute(diesel::migration::CREATE_MIGRATIONS_TABLE)
            .map(|()| 0)
    }
}
