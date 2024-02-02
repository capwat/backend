use quote::ToTokens;
use syn::Token;

use crate::derive::base_input::{GlobalAttrParse, VariantAttrParse};
use crate::utils::{self, AttributeValue};

#[derive(Clone)]
pub struct Attr {
    pub(super) message: Option<syn::LitStr>,
    pub(super) subcode: Option<u64>,
    pub(super) manual_deserialize: Option<()>,
    pub(super) manual_serialize: Option<()>,
}

impl GlobalAttrParse for Attr {
    fn from_ast(ctx: &utils::Context, variant: &syn::DeriveInput) -> Self {
        let mut message = AttributeValue::new(ctx, "message");
        let mut manual_deserialize =
            AttributeValue::new(ctx, "manual_deserialize");

        let mut manual_serialize = AttributeValue::new(ctx, "manual_serialize");
        let subcode = AttributeValue::new(ctx, "subcode");

        let attrs = variant.attrs.iter().filter(|v| {
            v.path().is_ident("error") && !matches!(&v.meta, syn::Meta::List(meta) if meta.tokens.is_empty())
        });
        for attr in attrs {
            let result = attr.parse_nested_meta(|meta| {
                let meta_path = &meta.path;
                if meta_path.is_ident("message") {
                    if let Some(content) =
                        utils::get_lit_str(ctx, "message", &meta)?
                    {
                        message.set(meta_path, content);
                    }
                } else if meta_path.is_ident("manual_deserialize") {
                    manual_deserialize.set(meta_path, ());
                } else if meta_path.is_ident("manual_serialize") {
                    manual_serialize.set(meta_path, ());
                } else {
                    let path = meta_path
                        .to_token_stream()
                        .to_string()
                        .replace(' ', "");

                    return Err(meta.error(format_args!(
                        "unknown field attribute `{path}`"
                    )));
                }
                Ok(())
            });
            if let Err(err) = result {
                ctx.error(err);
            }
        }

        Self {
            message: message.get(),
            subcode: subcode.get(),
            manual_deserialize: manual_deserialize.get(),
            manual_serialize: manual_serialize.get(),
        }
    }
}

impl VariantAttrParse for Attr {
    fn from_ast(ctx: &crate::utils::Context, variant: &syn::Variant) -> Self {
        let mut message = AttributeValue::new(ctx, "message");
        let mut subcode = AttributeValue::new(ctx, "subcode");

        let attrs = variant.attrs.iter().filter(|v| {
            v.path().is_ident("error") && !matches!(&v.meta, syn::Meta::List(meta) if meta.tokens.is_empty())
        });
        for attr in attrs {
            let result = attr.parse_nested_meta(|meta| {
                let meta_path = &meta.path;
                if meta_path.is_ident("message") {
                    if let Some(content) =
                        utils::get_lit_str(ctx, "message", &meta)?
                    {
                        message.set(meta_path, content);
                    }
                } else if meta_path.is_ident("subcode") {
                    meta.input.parse::<Token![=]>()?;
                    if let Some(value) =
                        utils::get_lit_int(ctx, "error", "subcode", &meta)?
                    {
                        subcode.set(meta_path, value);
                    }
                } else {
                    let path = meta_path
                        .to_token_stream()
                        .to_string()
                        .replace(' ', "");

                    return Err(meta.error(format_args!(
                        "unknown field attribute `{path}`"
                    )));
                }
                Ok(())
            });
            if let Err(err) = result {
                ctx.error(err);
            }
        }

        Self {
            message: message.get(),
            subcode: subcode.get(),
            manual_deserialize: None,
            manual_serialize: None,
        }
    }
}
