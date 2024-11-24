#![feature(let_chains)]
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod derive;

#[proc_macro_derive(ConfigParts, attributes(config))]
pub fn config_parts_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match self::derive::config_parts::expand(input) {
        Ok(okay) => okay,
        Err(error) => error.into_compile_error(),
    }
    .into()
}
