use chrono::NaiveDateTime;
use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::expression::AsExpression;
use diesel::internal::derives::as_expression::Bound;
use diesel::query_builder::bind_collector::RawBytesBindCollector;
use diesel::serialize::ToSql;
use diesel::sql_types::{HasSqlType, Timestamp as SqlTimestamp};
use diesel::Queryable;

use super::Timestamp;

impl AsExpression<SqlTimestamp> for Timestamp {
    type Expression = Bound<SqlTimestamp, NaiveDateTime>;

    fn as_expression(self) -> Self::Expression {
        let value = self.0.naive_utc();
        Bound::new(value)
    }
}

impl<D: Backend, T> FromSql<T, D> for Timestamp
where
    NaiveDateTime: FromSql<T, D>,
    D: HasSqlType<T>,
{
    fn from_sql(
        bytes: <D as Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        let value = NaiveDateTime::from_sql(bytes)?;
        Ok(value.into())
    }
}

impl<D, T> ToSql<T, D> for Timestamp
where
    NaiveDateTime: ToSql<T, D>,
    D: HasSqlType<T>,
    D: for<'c> Backend<BindCollector<'c> = RawBytesBindCollector<D>>,
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, D>,
    ) -> diesel::serialize::Result {
        let value = self.0.naive_utc();
        <NaiveDateTime as ToSql<T, D>>::to_sql(&value, &mut out.reborrow())
    }
}

impl<D: Backend, T> Queryable<T, D> for Timestamp
where
    NaiveDateTime: diesel::deserialize::FromStaticSqlRow<T, D>,
    D: HasSqlType<T>,
{
    type Row = NaiveDateTime;

    fn build(row: Self::Row) -> diesel::deserialize::Result<Self> {
        Ok(row.into())
    }
}

impl diesel::query_builder::QueryId for Timestamp {
    type QueryId = Self;
}
