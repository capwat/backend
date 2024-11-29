use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, ItemFn};

pub fn apply(tokens: TokenStream) -> syn::Result<TokenStream> {
    let mut input = syn::parse2::<ItemFn>(tokens)?;
    if input.sig.asyncness.is_none() {
        return Err(syn::Error::new(
            input.sig.span(),
            "all functions with #[postgres_query_test] must be async",
        ));
    }

    let test_ident = input.sig.ident;
    let inner_ident = syn::Ident::new("inner", test_ident.span());
    input.sig.ident = inner_ident.clone();

    Ok(quote! {
        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn #test_ident() {
            let vfs = ::capwat_vfs::Vfs::new_std();
            ::capwat_utils::env::load_dotenv(&vfs).ok();
            ::capwat_tracing::init_for_tests();

            #input

            if let Err(error) = #inner_ident().await {
                panic!("{error:#}");
            }
        }
    })
}
