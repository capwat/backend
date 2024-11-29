#![feature(let_chains)]
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod attrs;
mod derive;
mod macros;

#[proc_macro]
pub fn define_error_category(input: TokenStream) -> TokenStream {
    match self::macros::define_error_category::apply(input.into()) {
        Ok(okay) => okay,
        Err(error) => error.into_compile_error(),
    }
    .into()
}

// #[proc_macro_derive(ErrorSubcategory, attributes(category))]
// pub fn capwat_error_category_subcode(input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input as DeriveInput);
//     match self::derive::error_subcategory::expand(input) {
//         Ok(okay) => okay,
//         Err(error) => error.into_compile_error(),
//     }
//     .into()
// }

// #[proc_macro_attribute]
// pub fn capwat_error_category(_args: TokenStream, input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input as DeriveInput);
//     match self::attrs::error_category::expand(input) {
//         Ok(okay) => okay,
//         Err(error) => error.into_compile_error(),
//     }
//     .into()
// }

#[proc_macro_derive(ConfigParts, attributes(config))]
pub fn config_parts_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match self::derive::config_parts::expand(input) {
        Ok(okay) => okay,
        Err(error) => error.into_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn postgres_query_test(_args: TokenStream, item: TokenStream) -> TokenStream {
    match self::attrs::postgres_query_test::apply(item.into()) {
        Ok(okay) => okay,
        Err(error) => error.into_compile_error(),
    }
    .into()
}
