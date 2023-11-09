use syn::{parse_macro_input, DeriveInput};

mod expand;
mod internals;

#[proc_macro_derive(Validate, attributes(validate))]
pub fn derive_validate(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    expand::derive_validate(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
