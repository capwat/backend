use super::{Id, Marker};
use diesel::{pg::Pg, serialize::ToSql, sql_types::BigInt};
use std::error::Error;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("i64 out of bounds ({0})")]
struct OutOfBounds(u64);

#[derive(Debug, Error)]
#[error("expected positive i64, got negative ({0})")]
struct NegativeNumber(i64);

#[derive(Debug, Error)]
#[error("unexpected id returned zero")]
struct Zero;

fn to_i64(value: u64) -> Result<i64, OutOfBounds> {
    value.try_into().map_err(|_| OutOfBounds(value))
}

fn from_raw<T: Marker>(
    value: i64,
) -> Result<Id<T>, Box<dyn Error + Send + Sync>> {
    let value =
        value.try_into().map_err(|_| Box::new(NegativeNumber(value)))?;

    let value = Id::<T>::new_checked(value).ok_or_else(|| Box::new(Zero))?;
    Ok(value)
}

impl<T: Marker> diesel::serialize::ToSql<BigInt, Pg> for Id<T> {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        let value = to_i64(self.value.get())?;
        <i64 as ToSql<BigInt, diesel::pg::Pg>>::to_sql(
            &value,
            &mut out.reborrow(),
        )
    }
}

impl<T: Marker> diesel::expression::AsExpression<BigInt> for Id<T> {
    type Expression =
        diesel::internal::derives::as_expression::Bound<BigInt, i64>;

    fn as_expression(self) -> Self::Expression {
        // let's see if it can crash
        let value = to_i64(self.get()).expect("hi");
        diesel::internal::derives::as_expression::Bound::new(value)
    }
}

impl<ST, DB, T: Marker> diesel::deserialize::FromSql<ST, DB> for Id<T>
where
    i64: diesel::deserialize::FromSql<ST, DB>,
    DB: diesel::backend::Backend,
    DB: diesel::sql_types::HasSqlType<ST>,
{
    fn from_sql(
        raw: DB::RawValue<'_>,
    ) -> ::std::result::Result<Self, Box<dyn ::std::error::Error + Send + Sync>>
    {
        diesel::deserialize::FromSql::<ST, DB>::from_sql(raw).and_then(from_raw)
    }
}

impl<ST, DB, T: Marker> diesel::deserialize::Queryable<ST, DB> for Id<T>
where
    i64: diesel::deserialize::FromStaticSqlRow<ST, DB>,
    DB: diesel::backend::Backend,
    DB: diesel::sql_types::HasSqlType<ST>,
{
    type Row = i64;

    fn build(row: Self::Row) -> diesel::deserialize::Result<Self> {
        from_raw(row)
    }
}

impl<T: Marker + 'static> diesel::query_builder::QueryId for Id<T> {
    type QueryId = Self;
}
