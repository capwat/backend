use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, ItemFn};

pub fn apply(tokens: TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse2::<ItemFn>(tokens)?;
    if input.sig.asyncness.is_some() {
        return Err(syn::Error::new(
            input.sig.span(),
            "all functions with #[main] must be not async",
        ));
    }

    let caller_ident = &input.sig.ident;
    Ok(quote! {
        fn main() -> std::process::ExitCode {
            #input

            if let Err(error) = #caller_ident() {
                eprintln!("{error:#?}");
                std::process::ExitCode::FAILURE
            } else {
                std::process::ExitCode::SUCCESS
            }
        }
    })
}
