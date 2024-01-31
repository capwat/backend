use crate::internal::Sealed;

macro_rules! markers {
    ($(
        $( [features = [$feature:literal]] )?
        $( #[$meta:meta] )*
        $ident:ident,
    )*) => {$(
        $( #[cfg(feature = $feature)] )?
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $ident;

        $( #[cfg(feature = $feature)] )?
        $( #[$meta] )*
        impl Sealed for $ident {}

        $( #[cfg(feature = $feature)] )?
        $( #[$meta] )*
        impl Marker for $ident {}
    )*};
}

markers! {
  AnyMarker,
  UserMarker,
  [features = ["server_impl"]]
  SecretMarker,
}

/// This trait represents a marker restricting all objects to
/// from using it as a generic in [`Id`] object.
///
/// [`Id`]: super::Id
pub trait Marker: Sealed {}
