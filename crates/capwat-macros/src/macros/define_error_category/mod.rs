use proc_macro2::TokenStream;
use quote::quote;

mod defs;
mod printer;

pub fn apply(tokens: TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse2::<defs::Input>(tokens)?;

    let enum_definition = printer::EnumPrinter::new(&input);
    let raw_error_code_definition = printer::ErrorCodePrinter::new(&input);

    let deserialize_impl = printer::DeserializeImpl::new(&input);
    let serialize_impl = printer::SerializeImpl::new(&input);

    Ok(quote! {
        #enum_definition
        #raw_error_code_definition

        #deserialize_impl
        #serialize_impl
    })
}
