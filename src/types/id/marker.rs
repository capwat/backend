use crate::internal::Sealed;

macro_rules! markers {
  { $( $ident:ident, )* } => {$(
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct $ident;
    impl Sealed for $ident {}
    impl Marker for $ident {}
  )*};
}

markers! {
  AnyMarker,
  UserMarker,
}

/// This trait represents a marker restricting all objects to
/// from using it as a generic in [Id] object.
pub trait Marker: Sealed {}
