use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::DeriveInput;

mod hir;
mod input;

use self::input::Attr;
use crate::derive::base_input::EnumInput;
use crate::utils::Context;

#[cfg(test)]
mod tests;

pub fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let ctx = Context::new();
    let Some(input) = EnumInput::<Attr>::from_derive(&ctx, input) else {
        return Err(ctx.check_errors().unwrap_err());
    };

    let Some(input) = input.transform(&ctx) else {
        return Err(ctx.check_errors().unwrap_err());
    };

    ctx.check_errors()?;
    Ok(expand_input(&input))
}

fn generate_left_hand_side_pat<'a>(
    variant: &hir::Variant<'a>,
    tokens: &mut TokenStream,
    ignore_fields: bool,
) {
    let name = &variant.original.ident;
    tokens.extend(quote!(Self::#name));

    let has_inner_type = match &variant.r#type {
        hir::VariantType::Category { is_newtype, .. } => *is_newtype,
        hir::VariantType::Unknown => true,
    };

    if has_inner_type && ignore_fields {
        tokens.extend(quote!((..)));
    } else if has_inner_type {
        tokens.extend(quote!((inner)));
    }

    tokens.extend(quote!(=>));
}

// impl Error {
//      fn _has_data(&self) -> bool {
//          match self {
//              Self::Bar => false,
//              Self::Foo(n) => crate::error::SerializeCategory::has_data(n),
//              Self::Unknown(n) => crate::error::SerializeCategory::has_data(n),
//          }
//      }
//
//      // due to limitations with Rust (object safety), we need to actually make a
//      // function that uses SerializeMap to attempt to serialize `data` field
//      fn _serialize_data<S: ::serde::Serializer>(&self, map: &mut S::SerializeMap) -> ::std::result::Result<(), S::Error> {
//          // taking advantage of object safety problem :)
//          struct SerializeValue<'a, C: crate::error::SerializeCategory>(&'a C);
//
//          impl<'a, C: crate::error::SerializeCategory> ::serde::Serialize for SerializeValue<'a, C>
//              fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//              where
//                  S: serde::Serializer,
//              {
//                  self.0.serialize_data(serializer)
//              }
//          }
//
//          match self {
//              Self::Bar => panic!("Bar variant should not be serialized"),
//              Self::Foo(inner) => map.serialize_value(&SerializeValue(inner))?,
//              Self::Unknown(inner) => map.serialize_value(&SerializeValue(inner))?,
//          }
//      }
// }
fn expand_value_fns<'a>(input: &hir::Input<'a>) -> TokenStream {
    let mut has_data_tokens = TokenStream::new();
    let mut internal_serialize_tokens = TokenStream::new();

    for variant in input.variants.iter() {
        generate_left_hand_side_pat(variant, &mut has_data_tokens, false);
        generate_left_hand_side_pat(
            variant,
            &mut internal_serialize_tokens,
            false,
        );

        let has_field = match &variant.r#type {
            hir::VariantType::Category { is_newtype, .. } => *is_newtype,
            hir::VariantType::Unknown => true,
        };

        if has_field {
            has_data_tokens.extend(quote!(
                crate::error::SerializeCategory::has_data(inner)
            ));

            internal_serialize_tokens
                .extend(quote!(map.serialize_value(&SerializeValue(inner))));
        } else {
            has_data_tokens.extend(quote!(false));

            let name = &variant.original.ident;
            let panic_msg = format!("{name} variant should not be serialized");
            let panic_msg = syn::LitStr::new(&panic_msg, Span::call_site());
            internal_serialize_tokens.extend(quote!(panic!(#panic_msg)));
        }

        has_data_tokens.extend(quote!(,));
        internal_serialize_tokens.extend(quote!(,));
    }

    quote! {
        fn _has_data(&self) -> bool {
            match self {
                #has_data_tokens
            }
        }

        #[doc(hidden)]
        fn _serialize_data<S: ::serde::Serializer>(&self, map: &mut S::SerializeMap) -> ::std::result::Result<(), S::Error> {
            use ::serde::ser::SerializeMap;

            struct SerializeValue<'a, C: crate::error::SerializeCategory>(&'a C);

            impl<'a, C: crate::error::SerializeCategory> ::serde::Serialize for SerializeValue<'a, C> {
                fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
                where
                    S: ::serde::Serializer,
                {
                    self.0.serialize_data(serializer)
                }
            }

            match self {
                #internal_serialize_tokens
            }
        }
    }
}

// impl Error {
//      pub fn code(&self) -> u64 {
//          match self {
//              Self::Bar => 2,
//              Self::Foo(..) => 3,
//              Self::Unknown(n) => n.code,
//          }
//      }
//
//      fn _has_subcode(&self) -> Option<u64> {
//          match self {
//              Self::Bar => false,
//              Self::Foo(n) => crate::error::Category::has_subcode(n),
//              Self::Unknown(n) => crate::error::Category::has_subcode(n),
//          }
//      }
//
//      pub fn subcode(&self) -> Option<u64> {
//          match self {
//              Self::Bar => None,
//              Self::Foo(n) => crate::error::Category::subcode(n),
//              Self::Unknown(n) => crate::error::Category::subcode(n),
//          }
//      }
// }
fn expand_code_fns<'a>(input: &hir::Input<'a>) -> TokenStream {
    let mut code_tokens = TokenStream::new();
    let mut has_subcode_tokens = TokenStream::new();
    let mut subcode_tokens = TokenStream::new();

    for variant in input.variants.iter() {
        generate_left_hand_side_pat(variant, &mut has_subcode_tokens, false);
        generate_left_hand_side_pat(variant, &mut subcode_tokens, false);

        // code expansion
        match &variant.r#type {
            hir::VariantType::Category { code, is_newtype, .. } => {
                generate_left_hand_side_pat(variant, &mut code_tokens, true);
                code_tokens.extend(quote!(#code));
                if *is_newtype {
                    has_subcode_tokens.extend(quote!(
                        crate::error::Category::has_subcode(inner)
                    ));
                    subcode_tokens
                        .extend(quote!(crate::error::Category::subcode(inner)));
                } else {
                    has_subcode_tokens.extend(quote!(false));
                    subcode_tokens.extend(quote!(None));
                }
            },
            hir::VariantType::Unknown => {
                generate_left_hand_side_pat(variant, &mut code_tokens, false);
                code_tokens.extend(quote!(inner.code));
                has_subcode_tokens
                    .extend(quote!(crate::error::Category::has_subcode(inner)));
                subcode_tokens
                    .extend(quote!(crate::error::Category::subcode(inner)));
            },
        }

        code_tokens.extend(quote!(,));
        has_subcode_tokens.extend(quote!(,));
        subcode_tokens.extend(quote!(,));
    }

    quote! {
        #[must_use]
        pub fn code(&self) -> u64 {
            match self {
                #code_tokens
            }
        }

        fn _has_subcode(&self) -> bool {
            match self {
                #has_subcode_tokens
            }
        }

        #[must_use]
        pub fn subcode(&self) -> Option<u64> {
            match self {
                #subcode_tokens
            }
        }
    }
}

// impl Error {
//      pub const BAR_CODE: u64 = 1;
//      pub const FOO_CODE: u64 = 2;
//      fn _has_data(&self) -> bool {
//          match self {
//              Self::Bar => false,
//              Self::Foo(n) => crate::error::SerializeCategory::has_data(n),
//              Self::Unknown(n) => crate::error::SerializeCategory::has_data(n),
//          }
//      }
//
//      // due to limitations with Rust (object safety), we need to actually make a
//      // function that uses SerializeMap to attempt to serialize `data` field
//      fn _serialize_data<S: ::serde::Serializer>(&self, map: &mut S::SerializeMap) -> ::std::result::Result<(), S::Error> {
//          // taking advantage of object safety problem :)
//          struct SerializeValue<'a, C: crate::error::SerializeCategory>(&'a C);
//
//          impl<'a, C: crate::error::SerializeCategory> ::serde::Serialize for SerializeValue<'a, C>
//              fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//              where
//                  S: serde::Serializer,
//              {
//                  self.0.serialize_data(serializer)
//              }
//          }
//
//          match self {
//              Self::Bar => panic!("Bar variant should not be serialized"),
//              Self::Foo(inner) => map.serialize_value(&SerializeValue(inner))?,
//              Self::Unknown(inner) => map.serialize_value(&SerializeValue(inner))?,
//          }
//      }
// }
fn expand_const_codes<'a>(input: &hir::Input<'a>) -> TokenStream {
    let mut body = TokenStream::new();
    for variant in &input.variants {
        use heck::ToShoutySnakeCase;

        let name = variant.original.ident.to_string();
        let name = format!("{}_CODE", name.to_shouty_snake_case());
        let name = syn::Ident::new(&name, variant.original.ident.span());

        if let hir::VariantType::Category { code, .. } = &variant.r#type {
            body.extend(quote! {
                pub const #name: u64 = #code;
            });
        }
    }

    body
}

fn expand_serde_de_visitor<'a>(input: &hir::Input<'a>) -> TokenStream {
    let mut body = TokenStream::new();
    for variant in &input.variants {
        let hir::VariantType::Category { code, is_newtype, .. } =
            &variant.r#type
        else {
            continue;
        };

        let name = &variant.original.ident;
        body.extend(quote!(#code =>));

        if *is_newtype {
            body.extend(quote!({
                let value: _ = <_ as crate::error::DeserializeCategory>::deserialize(
                    self.subcode,
                    data.take(),
                )?;
                match value {
                    ::either::Either::Left(data) => {
                        return ::std::result::Result::Ok(crate::error::Error::#name(Box::new(data)))
                    },
                    ::either::Either::Right(n) => data = n,
                }
            }));
        } else {
            // Check if data is Some because it can be that this variant is outdated
            body.extend(quote!({
                if data.is_none() {
                    return ::std::result::Result::Ok(crate::error::Error::#name);
                }
            }));
        }
    }

    // Based from: https://github.com/twilight-rs/twilight/blob/main/twilight-model/src/gateway/event/gateway.rs#L180-L347
    // Licensed under ISC (Internet Systems Consortium) license
    quote! {
        #[doc(hidden)]
        #[derive(Debug, ::serde::Deserialize, PartialEq, Eq)]
        #[serde(field_identifier)]
        enum Field {
            #[serde(rename = "code")]
            Code,
            #[serde(rename = "subcode")]
            Subcode,
            #[serde(rename = "message")]
            Message,
            #[serde(rename = "data")]
            Data,
        }

        struct ErrorVisitor<'a> {
            code: u64,
            subcode: Option<u64>,
            message: ::std::borrow::Cow<'a, str>,
        }

        impl<'a> ErrorVisitor<'a> {
            pub fn new(code: u64, subcode: Option<u64>, message: ::std::borrow::Cow<'a, str>) -> Self {
                Self { code, subcode, message }
            }
        }

        impl ErrorVisitor<'_> {
            fn field<'de, T: ::serde::de::Deserialize<'de>, V: ::serde::de::MapAccess<'de>>(
                map: &mut V,
                field: Field,
            ) -> Result<::std::option::Option<T>, V::Error> {
                let mut found = None;
                loop {
                    match map.next_key::<Field>() {
                        Ok(Some(key)) if key == field => {
                            found = Some(map.next_value()?)
                       },
                        Ok(Some(_)) | Err(_) => {
                            map.next_value::<::serde::de::IgnoredAny>()?;
                            continue;
                        },
                        Ok(None) => {
                            break;
                        },
                    }
                }
                Ok(found)
            }
        }


        impl<'a, 'de> ::serde::de::Visitor<'de> for ErrorVisitor<'a> {
            type Value = crate::error::Error;

            fn expecting(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str("Capwat error")
            }

            fn visit_map<A>(self, mut map: A) -> ::std::result::Result<Self::Value, A::Error>
            where
                A: ::serde::de::MapAccess<'de>,
            {
                let mut data: Option<::serde_json::Value> =
                    Self::field(&mut map, Field::Data)?;

                match self.code {
                    #body
                    _ => {},
                };

                ::std::result::Result::Ok(crate::error::Error::Unknown(Box::new(crate::error::Unknown {
                    code: self.code,
                    subcode: self.subcode,
                    message: self.message.to_string(),
                    data,
                })))
            }
        }
    }
}

// pub enum ErrorCode {
//     Hello,
//     Internal,
// }
fn generate_code_kinds<'a>(input: &hir::Input<'a>) -> TokenStream {
    let mut variants = TokenStream::new();
    let mut other_body = TokenStream::new();
    let mut to_code_body = TokenStream::new();
    let mut from_code_matches = TokenStream::new();

    let name = &input.original.ident;
    let (impl_generics, ty_generics, where_clause) =
        input.original.generics.split_for_impl();

    let kind_name = format!("{}Code", input.original.ident);
    let kind_name = syn::Ident::new(&kind_name, Span::call_site());

    for variant in input.variants.iter() {
        let hir::VariantType::Category { code, .. } = &variant.r#type else {
            continue;
        };

        let variant_name = &variant.original.ident;
        generate_left_hand_side_pat(variant, &mut other_body, false);

        variants.extend(quote!(#variant_name,));
        other_body.extend(quote!(#kind_name::#variant_name,));
        to_code_body.extend(quote!(Self::#variant_name => #code,));
        from_code_matches.extend(quote!(#code => Self::#variant_name,));
    }

    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum #kind_name {
            #variants
            Unknown(u64),
        }

        impl #kind_name {
            #[must_use]
            pub const fn from_code(code: u64) -> Self {
                match code {
                    #from_code_matches
                    _ => Self::Unknown(code),
                }
            }

            #[must_use]
            pub const fn code(&self) -> u64 {
                match self {
                    #to_code_body
                    Self::Unknown(n) => *n,
                }
            }
        }

        impl #impl_generics #name #ty_generics #where_clause {
            #[must_use]
            pub const fn code_kind(&self) -> #kind_name {
                match self {
                    #other_body
                    #name::Unknown(data) => #kind_name::Unknown(data.code),
                }
            }
        }
    }
}

// impl Error {
//      fn _make_message(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//          use std::fmt::Display;
//          match self {
//              Self::Bar => f.write_str("Hi!"),
//              Self::Foo(err) => crate::error::Category::message(err, f),
//              Self::Unknown(err) => crate::error::Category::message(err, f),
//          }
//      }
// }
fn expand_msg_block<'a>(input: &hir::Input<'a>) -> TokenStream {
    let mut block = TokenStream::new();
    for variant in input.variants.iter() {
        generate_left_hand_side_pat(variant, &mut block, false);

        match &variant.r#type {
            hir::VariantType::Category { is_newtype, message, .. } => {
                if *is_newtype {
                    block.extend(quote!(
                        crate::error::CategoryMessage::message(inner, f)
                    ));
                } else {
                    block.extend(quote!(f.write_str(#message)));
                }
            },
            hir::VariantType::Unknown => {
                block.extend(quote!(crate::error::CategoryMessage::message(
                    inner, f
                )));
            },
        };

        block.extend(quote!(,));
    }
    block
}

fn expand_input<'a>(input: &hir::Input<'a>) -> TokenStream {
    let msg_block = expand_msg_block(input);
    let code_fns = expand_code_fns(input);
    let value_fns = expand_value_fns(input);
    let code_consts = expand_const_codes(input);
    let visitor = expand_serde_de_visitor(input);
    let kinds = generate_code_kinds(input);

    let name = &input.original.ident;
    let (impl_generics, ty_generics, where_clause) =
        input.original.generics.split_for_impl();

    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #code_fns
            #value_fns
            #code_consts

            fn _make_message(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                use std::fmt::Display;
                match self {
                    #msg_block
                }
            }
        }

        #visitor
        #kinds
    }
}
