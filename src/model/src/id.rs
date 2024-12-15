use thiserror::Error;

macro_rules! newtypes {
    {
        $( $Ident:ident: $ty:ty, )*
    } => {$(
        #[derive(Debug, serde::Deserialize, serde::Serialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[serde(transparent)]
        pub struct $Ident(pub $ty);

        impl From<$ty> for $Ident {
            fn from(value: $ty) -> Self {
                Self(value)
            }
        }

        impl<'q> sqlx::Encode<'q, sqlx::Postgres> for $Ident
        where
            $ty: sqlx::Encode<'q, sqlx::Postgres>,
        {
            fn encode_by_ref(
                &self,
                buf: &mut <sqlx::Postgres as sqlx::Database>::ArgumentBuffer<'q>,
            ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
                <$ty>::encode_by_ref(&self.0, buf)
            }
        }

        impl<'r> sqlx::Decode<'r, sqlx::Postgres> for $Ident
        where
            $ty: sqlx::Decode<'r, sqlx::Postgres>,
        {
            fn decode(
                value: <sqlx::Postgres as sqlx::Database>::ValueRef<'r>,
            ) -> Result<Self, sqlx::error::BoxDynError> {
                let inner = <$ty>::decode(value)?;
                if inner < 0 {
                    return Err(Box::new(NegativeId));
                }

                Ok(Self(inner))
            }
        }

        impl<DB: sqlx::Database> sqlx::Type<DB> for $Ident
        where
            $ty: sqlx::Type<DB>,
        {
            fn type_info() -> <DB as sqlx::Database>::TypeInfo {
                <$ty>::type_info()
            }
        }
    )*};
}

#[derive(Debug, Error)]
#[error("unexpected ID has a negative value")]
struct NegativeId;

newtypes! {
    UserId: i64,
    UserKeysId: i64,
    FollowerId: i64,

    PostId: i64,

    InstanceId: i32,
}
