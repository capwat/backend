use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Error, attributes(error))]
pub fn error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    capwat_types_derive_lib::derive::error::expand(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(CategoryError, attributes(error))]
pub fn category_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    capwat_types_derive_lib::derive::category_error::expand(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
