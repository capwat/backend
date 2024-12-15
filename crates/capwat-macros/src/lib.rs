#![feature(let_chains)]
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod attrs;
mod derive;
mod macros;

mod utils;

#[proc_macro]
pub fn define_error_category(input: TokenStream) -> TokenStream {
    match self::macros::define_error_category::apply(input.into()) {
        Ok(okay) => okay,
        Err(error) => error.into_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn main(_args: TokenStream, item: TokenStream) -> TokenStream {
    match self::attrs::main::apply(item.into()) {
        Ok(okay) => okay,
        Err(error) => error.into_compile_error(),
    }
    .into()
}

#[proc_macro_derive(ConfigParts, attributes(config))]
pub fn config_parts_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match self::derive::config_parts::expand(input) {
        Ok(okay) => okay,
        Err(error) => error.into_compile_error(),
    }
    .into()
}

#[proc_macro_derive(SeaTable, attributes(sea_table))]
pub fn sea_table_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match self::derive::sea_table::expand(input) {
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

#[proc_macro_attribute]
pub fn server_test(_args: TokenStream, item: TokenStream) -> TokenStream {
    match self::attrs::server_test::apply(item.into()) {
        Ok(okay) => okay,
        Err(error) => error.into_compile_error(),
    }
    .into()
}
