use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::DeriveInput;

mod hir;
mod input;

use self::input::Attr;
use crate::derive::base_input::Input;
use crate::utils::Context;

pub fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let ctx = Context::new();
    let Some(input) = Input::<Attr>::from_derive(&ctx, input) else {
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

    let has_field = variant.is_newtype;
    if has_field && ignore_fields {
        tokens.extend(quote!((..)));
    } else if has_field {
        tokens.extend(quote!((inner)));
    }

    tokens.extend(quote!(=>));
}

fn expand_const_codes<'a>(input: &hir::Input<'a>) -> TokenStream {
    let Some(variants) = input.variants.as_ref() else { unreachable!() };

    let name = &input.original.ident;
    let (impl_generics, ty_generics, where_clause) =
        input.original.generics.split_for_impl();

    let mut body = TokenStream::new();
    for variant in variants {
        use heck::ToShoutySnakeCase;

        let name = variant.original.ident.to_string();
        let name = format!("{}_CODE", name.to_shouty_snake_case());
        let name = syn::Ident::new(&name, variant.original.ident.span());

        let subcode = variant.subcode;
        body.extend(quote! {
            pub const #name: u64 = #subcode;
        });
    }

    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #body
        }
    }
}

fn expand_category_impl<'a>(input: &hir::Input<'a>) -> TokenStream {
    let mut subcode_body = TokenStream::new();
    let has_subcode_body = if let Some(variants) = input.variants.as_ref() {
        for variant in variants {
            generate_left_hand_side_pat(variant, &mut subcode_body, true);
            let subcode = variant.subcode;
            subcode_body.extend(quote!(#subcode,));
        }
        subcode_body = quote! {
            Some(match self {
                #subcode_body
            })
        };
        quote!(true)
    } else {
        subcode_body = quote!(None);
        quote!(None)
    };

    quote! {
        fn subcode(&self) -> ::std::option::Option<u64> {
            #subcode_body
        }

        fn has_subcode(&self) -> bool {
            #has_subcode_body
        }
    }
}

fn expand_category_message_impl<'a>(input: &hir::Input<'a>) -> TokenStream {
    let name = &input.original.ident;
    let (impl_generics, ty_generics, where_clause) =
        input.original.generics.split_for_impl();

    let body = if let Some(variants) = input.variants.as_ref() {
        let mut block = TokenStream::new();
        for variant in variants {
            let has_field = variant.is_newtype;
            generate_left_hand_side_pat(variant, &mut block, !has_field);

            // Use the message literal or SubcategoryMessage?
            if has_field && variant.message.is_none() {
                block.extend(quote!(
                    crate::error::SubcategoryMessage::message(inner, f)
                ));
            }

            if let Some(message) = variant.message.as_ref() {
                block.extend(quote!(f.write_str(#message)));
            }

            block.extend(quote!(,));
        }

        quote! {
            match self {
                #block
            }
        }
    } else {
        let Some(message) = input.global_attr.message.as_ref() else {
            unreachable!()
        };
        quote!(f.write_str(#message))
    };

    quote! {
        impl #impl_generics crate::error::CategoryMessage for #name #ty_generics #where_clause {
            fn message(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                #body
            }
        }
    }
}

fn expand_deserialize_category_impl<'a>(input: &hir::Input<'a>) -> TokenStream {
    let Some(variants) = input.variants.as_ref() else { unreachable!() };

    let mut body = TokenStream::new();
    let name = &input.original.ident;
    let (impl_generics, ty_generics, where_clause) =
        input.original.generics.split_for_impl();

    for variant in variants {
        let subcode = variant.subcode;
        body.extend(quote!(Some(#subcode) => ));

        let has_field = variant.is_newtype;
        let name = &variant.original.ident;
        if has_field {
            body.extend(quote!({
                let data = data
                    .as_ref()
                    .ok_or_else(|| ::serde::de::Error::missing_field("data"))?;

                let value = <_ as ::serde::Deserialize>::deserialize(
                    data.into_deserializer(),
                )
                .map_err(::serde::de::Error::custom)?;

                ::either::Either::Left(Self::#name(Box::new(value)))
            }));
        } else {
            body.extend(quote!(Either::Left(Self::#name)));
        }

        body.extend(quote!(,));
    }

    quote! {
        impl #impl_generics crate::error::DeserializeCategory for #name #ty_generics #where_clause {
            fn deserialize<D: ::serde::de::Error>(
                subcode: ::std::option::Option<u64>,
                data: ::std::option::Option<::serde_json::Value>,
            ) -> ::std::result::Result<::either::Either<Self, ::std::option::Option<::serde_json::Value>>, D> {
                use serde::de::IntoDeserializer;
                Ok(match subcode {
                    #body
                    _ => ::either::Either::Right(data),
                })
            }
        }
    }
}

fn expand_serialize_category_impl<'a>(input: &hir::Input<'a>) -> TokenStream {
    let Some(variants) = input.variants.as_ref() else { unreachable!() };

    let mut body = TokenStream::new();
    let mut has_data_body = TokenStream::new();

    let name = &input.original.ident;
    let (impl_generics, ty_generics, where_clause) =
        input.original.generics.split_for_impl();

    for variant in variants {
        generate_left_hand_side_pat(variant, &mut body, false);
        generate_left_hand_side_pat(variant, &mut has_data_body, true);

        let has_field = variant.is_newtype;
        if has_field {
            body.extend(quote!(::serde::Serialize::serialize(
                &inner, serializer
            )));
        } else {
            let name = &variant.original.ident;
            let panic_msg = format!("{name} variant should not be serialized");
            let panic_msg = syn::LitStr::new(&panic_msg, Span::call_site());
            body.extend(quote!(panic!(#panic_msg)));
        }

        has_data_body.extend(quote!(#has_field,));
        body.extend(quote!(,));
    }

    quote! {
        impl #impl_generics crate::error::SerializeCategory for #name #ty_generics #where_clause {
            fn has_data(&self) -> bool {
                match self {
                    #has_data_body
                }
            }

            fn serialize_data<S: ::serde::Serializer>(
                &self,
                serializer: S,
            ) -> ::std::result::Result<S::Ok, S::Error> {
                match self {
                    #body
                }
            }
        }
    }
}

fn expand_input<'a>(input: &hir::Input<'a>) -> TokenStream {
    let category_impl = expand_category_impl(input);

    let name = &input.original.ident;
    let (impl_generics, ty_generics, where_clause) =
        input.original.generics.split_for_impl();

    let mut tokens = quote! {
        impl #impl_generics crate::error::Category for #name #ty_generics #where_clause {
            #category_impl
        }
    };
    tokens.extend(expand_category_message_impl(input));

    let manually_deserialize = input.global_attr.manual_deserialize.is_some();
    let manually_serialize = input.global_attr.manual_serialize.is_some();
    let is_enum = input.variants.is_some();

    // Auto-generated constant codes
    if is_enum {
        tokens.extend(expand_const_codes(input));
    }

    if is_enum && !manually_deserialize {
        let body = expand_deserialize_category_impl(input);
        tokens.extend(body);
    }

    if is_enum && !manually_serialize {
        let body = expand_serialize_category_impl(input);
        tokens.extend(body);
    }

    tokens
}
