use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod expand;

#[proc_macro_derive(Error, attributes(error))]
pub fn derive_error(input: TokenStream) -> TokenStream {
  expand::error(parse_macro_input!(input as DeriveInput))
    .unwrap_or_else(|e| e.into_compile_error())
    .into()
}
