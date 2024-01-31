use quote::ToTokens;
use syn::Token;

use crate::derive::base_input::VariantAttrParse;
use crate::utils::{self, AttributeValue};

#[derive(Clone)]
pub struct Attr {
    pub(super) message: Option<syn::LitStr>,
    pub(super) code: Option<u64>,
    pub(super) unknown: Option<()>,
}

impl VariantAttrParse for Attr {
    fn from_ast(ctx: &crate::utils::Context, variant: &syn::Variant) -> Self {
        let mut message = AttributeValue::new(ctx, "message");
        let mut code = AttributeValue::new(ctx, "code");
        let mut unknown = AttributeValue::new(ctx, "unknown");

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
                } else if meta_path.is_ident("code") {
                    meta.input.parse::<Token![=]>()?;
                    if let Some(value) =
                        utils::get_lit_int(ctx, "error", "code", &meta)?
                    {
                        code.set(meta_path, value);
                    }
                } else if meta_path.is_ident("unknown") {
                    unknown.set(meta_path, ());
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
            code: code.get(),
            unknown: unknown.get(),
        }
    }
}
