#[cfg(feature = "with_diesel")]
use diesel_derive_newtype::DieselNewType;

macro_rules! newtypes {
    {
        $( $Ident:ident: $ty:ty, )*
    } => {$(
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[cfg_attr(feature = "with_diesel", derive(DieselNewType))]
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

    PostId: i64,
    PostClusterId: i64,
    PostClusterKeyId: i64,
    PostClusterMemberId: i64,
    PostClusterMemberKeysId: i64,

    InstanceId: i32,
}
