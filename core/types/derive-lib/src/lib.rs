pub mod derive;
pub mod utils;

// #[test]
// fn that() {
//     use proc_macro2::TokenStream;
//     use std::str::FromStr;

//     let source = r#"
//     #[derive(Debug, Error)]
//     pub enum Error {
//         #[error(code = 1)]
//         #[error(message = "Internal server occurred. Please try again later.")]
//         Internal,
//         #[error(code = 2)]
//         #[error(
//             message = "This service is currently in read only mode. Please try again later."
//         )]
//         ReadonlyMode,
//         #[error(code = 3)]
//         #[error(message = "Not authenticated")]
//         NotAuthenticated,
//         #[error(code = 1)]
//         LoginUser(LoginUser),
//         #[error(unknown)]
//         Unknown(Unknown),
//     }
//     "#;

//     let source = TokenStream::from_str(source).unwrap();
//     let input = syn::parse2::<syn::DeriveInput>(source).unwrap();

//     let that = derive::error::expand(&input).unwrap();
//     panic!("{that}");
// }
