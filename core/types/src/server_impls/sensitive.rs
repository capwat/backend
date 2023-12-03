use crate::sensitive::Sensitive;

impl<'q, DB: sqlx::Database, T: sqlx::Decode<'q, DB>> sqlx::Decode<'q, DB>
  for Sensitive<T>
{
  fn decode(
    value: <DB as sqlx::database::HasValueRef<'q>>::ValueRef,
  ) -> Result<Self, sqlx::error::BoxDynError> {
    T::decode(value).map(Sensitive::new)
  }
}

impl<'q, DB: sqlx::Database, T: sqlx::Encode<'q, DB>> sqlx::Encode<'q, DB>
  for Sensitive<T>
{
  fn encode_by_ref(
    &self,
    buf: &mut <DB as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
  ) -> sqlx::encode::IsNull {
    self.as_ref().encode(buf)
  }
}

impl<DB: sqlx::Database, T: sqlx::Type<DB>> sqlx::Type<DB> for Sensitive<T> {
  fn compatible(ty: &<DB as sqlx::Database>::TypeInfo) -> bool {
    T::compatible(ty)
  }

  fn type_info() -> <DB as sqlx::Database>::TypeInfo {
    T::type_info()
  }
}
