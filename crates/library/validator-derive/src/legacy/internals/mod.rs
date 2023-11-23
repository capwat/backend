mod context;

pub use context::*;
pub mod input;
pub mod utils;

pub type ExpandResult = std::result::Result<proc_macro2::TokenStream, syn::Error>;
