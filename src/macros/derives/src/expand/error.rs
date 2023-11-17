use darling::{FromDeriveInput, FromVariant};
use proc_macro2::TokenStream;
use quote::{quote, spanned::Spanned};
use syn::{DeriveInput, Ident};

#[derive(Debug, FromVariant)]
#[darling(attributes(error))]
struct ErrorVariant {
  ident: Ident,
  fields: darling::ast::Fields<darling::util::Ignored>,
  message: Option<syn::LitStr>,
  subcode: syn::ExprPath,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(error))]
struct Error {
  ident: Ident,
  message: Option<syn::LitStr>,
  code: syn::ExprPath,
  data: darling::ast::Data<ErrorVariant, darling::util::Ignored>,
}

pub fn error(input: DeriveInput) -> syn::Result<TokenStream> {
  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
  let input_span = input.__span();
  let input = Error::from_derive_input(&input)?;

  let name = &input.ident;
  let primary_code = &input.code;

  let mut message_data_tokens = TokenStream::new();
  // let mut has_data_tokens = TokenStream::new();
  let mut subcode_tokens = TokenStream::new();
  let mut data_tokens = TokenStream::new();
  let mut de_tokens = TokenStream::new();

  match &input.data {
    darling::ast::Data::Enum(variants) => {
      if variants.len() == 0 {
        return Err(syn::Error::new(
          input_span,
          "#[derive(Error)] does not support any enums that have no any variants",
        ));
      }
      if input.message.is_some() {
        return Err(syn::Error::new(
          input_span,
          "#[error(message = ...)] in non-subcode type is not allowed",
        ));
      }

      for variant in variants {
        if variant.fields.is_empty() {
          if variant.message.is_none() {
            return Err(syn::Error::new(
              variant.ident.span(),
              "#[error(message = ...)] is required for unit variants",
            ));
          }
        } else {
          if !variant.fields.is_newtype() {
            return Err(syn::Error::new(
              variant.ident.span(),
              "#[derive(Error)] does not accept any struct data aside from newtype",
            ));
          }
          if variant.message.is_some() {
            return Err(syn::Error::new(
              variant.ident.span(),
              "#[error(message = ...)] must not be used if it is a newtype",
            ));
          }
        }

        // TODO: Cleanup and only require `subcode` attr if it is a unit type.
        let ident = &variant.ident;
        let subcode = &variant.subcode;
        let prefix;
        if variant.fields.is_newtype() {
          prefix = quote!( Self::#ident(n) );
          message_data_tokens.extend(quote! {
              #prefix => crate::error::Tertiary::message(n),
          });
          // has_data_tokens.extend(quote! {
          //     Self::#ident(..) => true,
          // });
          data_tokens.extend(quote! {
              #prefix => Some(::serde_value::to_value(n)?),
          });
          de_tokens.extend(quote! {
              #subcode => {
                  if let Some(value) = value {
                      value
                          .deserialize_into()
                          .map_err(crate::error::PrimaryDeserializeError::Custom)
                          .map(Self::#ident)
                  } else {
                      Err(crate::error::PrimaryDeserializeError::MissingData)
                  }
              },
          });
        } else {
          prefix = quote!( Self::#ident );
          let message = variant.message.as_ref().unwrap();
          message_data_tokens.extend(quote! {
              #prefix => ::std::borrow::Cow::Borrowed(#message),
          });
          // has_data_tokens.extend(quote! {
          //     #prefix => false,
          // });
          data_tokens.extend(quote! {
              #prefix => None,
          });
          de_tokens.extend(quote! {
              #subcode => Ok(#prefix),
          });
        }
        subcode_tokens.extend(quote! {
            #prefix => ::std::option::Option::Some(&#subcode),
        });
      }
      subcode_tokens = quote! {
          match self {
              #subcode_tokens
          }
      };
      message_data_tokens = quote! {
          match self {
              #message_data_tokens
          }
      };
      // has_data_tokens = quote! {
      //     match self {
      //         #has_data_tokens
      //     }
      // };
      data_tokens = quote! {
          match self {
              #data_tokens
          }
      };
      de_tokens = quote! {
          if let Some(subcode) = subcode {
              match subcode {
                  #de_tokens
                  _ => Err(
                      crate::error::PrimaryDeserializeError::InvalidSubcode(
                          stringify!(#name),
                          subcode
                      )
                  ),
              }
          } else {
              Err(crate::error::PrimaryDeserializeError::MissingData)
          }
      };
    }
    darling::ast::Data::Struct(data) => {
      if data.is_unit() {
        subcode_tokens = quote!(None);
        // has_data_tokens = quote!(false);

        let message = input
          .message
          .ok_or_else(|| syn::Error::new(input_span, "#[error(message = ...)] is required"))?;
        message_data_tokens = quote!(::std::borrow::Cow::Borrowed(#message));
        data_tokens = quote!(None);
        de_tokens = quote!(Ok(Self));
      } else {
        return Err(syn::Error::new(
          input_span,
          "#[derive(Error)] with data is not supported",
        ));
      }
    }
  };

  Ok(quote! {
      impl #impl_generics crate::error::PrimaryCreator for #name #ty_generics #where_clause {
        fn name() -> &'static str {
          stringify!(#name)
        }

        fn code() -> &'static u32 {
            &#primary_code
        }

        #[allow(unused)]
          fn from_subcode(
              subcode: ::std::option::Option<u32>,
              value: ::std::option::Option<serde_value::Value>,
          ) -> ::std::result::Result<Self, crate::error::PrimaryDeserializeError>
          where
              Self: Sized
          {
              #de_tokens
          }
      }

      impl #impl_generics crate::error::Primary for #name #ty_generics #where_clause {
          fn subcode(&self) -> ::std::option::Option<&'static u32> {
              #subcode_tokens
          }

          fn message(&self) -> ::std::borrow::Cow<'static, str> {
              #message_data_tokens
          }

          // fn has_data(&self) -> bool {
          //     #has_data_tokens
          // }

          fn data(&self) -> ::std::result::Result<::std::option::Option<::serde_value::Value>, ::serde_value::SerializerError> {
              Ok(#data_tokens)
          }
      }
  })
}
