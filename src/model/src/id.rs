use diesel_derive_newtype::DieselNewType;

macro_rules! newtypes {
    {
        $( $Ident:ident: $ty:ty, )*
    } => {$(
        #[derive(Debug, serde::Deserialize, serde::Serialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, DieselNewType)]
        #[serde(transparent)]
        pub struct $Ident(pub $ty);

        impl From<$ty> for $Ident {
            fn from(value: $ty) -> Self {
                Self(value)
            }
        }
    )*};
}

newtypes! {
    UserId: i64,
    UserKeysId: i64,
    FollowerId: i64,

    PostId: i64,

    InstanceId: i32,
}
