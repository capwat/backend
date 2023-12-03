use crate::internal::Sealed;

macro_rules! markers {
  { $( $(#[$meta:meta] )* $ident:ident, )* } => {$(
    $( #[$meta] )*
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct $ident;
    $( #[$meta] )*
    impl Sealed for $ident {}
    $( #[$meta] )*
    impl Marker for $ident {}
  )*};
}

markers! {
  AnyMarker,
  UserMarker,
  #[cfg(feature = "server_impl")]
  SecretMarker,
}

/// This trait represents a marker restricting all objects to
/// from using it as a generic in [Id] object.
pub trait Marker: Sealed {}
