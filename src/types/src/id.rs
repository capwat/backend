use thiserror::Error;

#[derive(Debug, Error)]
#[error("all IDs must be positive")]
struct NegativeIdError;

#[derive(Debug, Error)]
#[error("an ID reached the 64-bit signed integer limit")]
struct OverflowError;

make_ids! {
  pub struct UserId(u64 => i64);
}

// TODO: convert this into snowflake smth
macro_rules! make_ids {
  { $( $( #[$Meta:meta] )* $Visibility:vis struct $Name:ident($Type:tt => $SqlType:tt); )* } => {$(
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize, serde::Serialize)]
    #[serde(transparent)]
    $Visibility struct $Name(pub $Type);

    #[cfg(feature = "server")]
    impl<'r> sqlx::Decode<'r, sqlx::Postgres> for $Name {
      fn decode(
        value: <sqlx::Postgres as sqlx::database::HasValueRef<'r>>::ValueRef,
      ) -> Result<Self, sqlx::error::BoxDynError> {
        let value = <$SqlType as sqlx::Decode<'r, sqlx::Postgres>>::decode(value)?;
        if value.is_negative() {
          return Err(Box::new(NegativeIdError));
        }
        Ok(Self(value.abs() as $Type))
      }
    }

    #[cfg(feature = "server")]
    impl<'q> sqlx::Encode<'q, sqlx::Postgres> for $Name {
      fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Postgres as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
      ) -> sqlx::encode::IsNull {
        const LIMIT: $Type = $SqlType::MAX as $Type;
        if self.0 > LIMIT {
          return sqlx::encode::IsNull::Yes;
        }
        <$SqlType as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(&(self.0 as $SqlType), buf)
      }
    }

    #[cfg(feature = "server")]
    impl sqlx::Type<sqlx::Postgres> for $Name {
      fn type_info() -> <sqlx::Postgres as sqlx::Database>::TypeInfo {
        <$SqlType as sqlx::Type<sqlx::Postgres>>::type_info()
      }
    }
  )*};
}
use make_ids;
