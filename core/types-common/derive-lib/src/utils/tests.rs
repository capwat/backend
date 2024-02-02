use proc_macro2::TokenStream;
use std::str::FromStr;

pub fn parse_input(input: &str) -> syn::DeriveInput {
    let source = TokenStream::from_str(input).expect("invalid source");
    syn::parse2::<syn::DeriveInput>(source).expect("invalid source")
}

pub fn extract_error_msg<T>(result: Result<T, syn::Error>) -> String {
    match result {
        Ok(..) => panic!("unexpected successful execution"),
        Err(e) => e.to_string(),
    }
}
