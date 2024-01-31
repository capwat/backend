use super::input::Attr;
use crate::derive::base_input;
use crate::utils::{self, Context};

pub struct Input<'a> {
    pub variants: Vec<Variant<'a>>,
    pub original: base_input::EnumInput<'a, Attr>,
}

pub struct Variant<'a> {
    pub r#type: VariantType,
    pub original: base_input::Variant<'a, Attr>,
}

pub enum VariantType {
    Category { code: u64, is_newtype: bool, message: Option<syn::LitStr> },
    Unknown,
}

impl<'a> base_input::EnumInput<'a, Attr> {
    pub fn transform(self, ctx: &Context) -> Option<Input<'a>> {
        let mut variants = Vec::new();
        let mut has_unknown_variant = false;
        for variant in self.variants.iter() {
            let variant = variant.clone().transform(ctx)?;
            if matches!(variant.r#type, VariantType::Unknown) {
                if has_unknown_variant {
                    ctx.spanned_error(
                        &self.original,
                        "#[error(unknown)] should not be used twice",
                    );
                    return None;
                }

                has_unknown_variant = true;
            }
            variants.push(variant);
        }
        if !has_unknown_variant {
            ctx.spanned_error(&self.original, "#[error(unknown)] must be used");
            return None;
        }
        Some(Input { variants, original: self })
    }
}

impl<'a> base_input::Variant<'a, Attr> {
    pub fn transform(self, ctx: &Context) -> Option<Variant<'a>> {
        let Some(fields) = utils::get_unnamed_variants(&self.original) else {
            ctx.spanned_error(
                &self.original,
                "Every error category must be either in unit or unnamed fields",
            );
            return None;
        };

        let attr = &self.attr;
        let r#type = if attr.unknown.is_some() {
            if attr.code.is_some() {
                ctx.spanned_error(&self.original, "#[error(code = ...)] should not be used with #[error(unknown)]");
                return None;
            }

            if attr.message.is_some() {
                ctx.spanned_error(&self.original, "#[error(message = ...)] should not be used with #[error(unknown)]");
                return None;
            }

            if fields.map(|v| v.len()).unwrap_or_default() != 1 {
                ctx.spanned_error(
                    &self.original,
                    "#[error(unknown)] variant must have one unnamed field",
                );
                return None;
            }

            VariantType::Unknown
        } else {
            let Some(code) = attr.code else {
                ctx.spanned_error(
                    &self.original,
                    "#[error(code = ...)] is required",
                );
                return None;
            };

            let len_variants = fields.map(|v| v.len()).unwrap_or(0);
            if len_variants == 0 && fields.is_some() {
                ctx.spanned_error(
                    &self.original,
                    "Every error category must be either a unit variant or has one unnamed field",
                );
                return None;
            }

            if len_variants > 1 {
                ctx.spanned_error(
                    &self.original,
                    "Every error category must be either a unit variant or has one unnamed field",
                );
                return None;
            }

            if len_variants != 0 && attr.message.is_some() {
                ctx.spanned_error(
                    &self.original,
                    "#[error(message = ...)] must be used in unit variants",
                );
                return None;
            }

            if len_variants == 0 && attr.message.is_none() {
                ctx.spanned_error(
                    &self.original,
                    "#[error(message = ...)] is required for unit variants",
                );
                return None;
            }

            VariantType::Category {
                code,
                is_newtype: len_variants == 1,
                message: attr.message.clone(),
            }
        };

        Some(Variant { r#type, original: self })
    }
}
