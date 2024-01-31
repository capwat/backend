use super::input::Attr;
use crate::derive::base_input;
use crate::utils::{self, Context};

pub struct Input<'a> {
    pub variants: Option<Vec<Variant<'a>>>,
    pub global_attr: Attr,
    pub original: base_input::Input<'a, Attr>,
}

pub struct Variant<'a> {
    pub subcode: u64,
    pub is_newtype: bool,
    pub message: Option<syn::LitStr>,
    pub original: base_input::Variant<'a, Attr>,
}

impl<'a> base_input::Input<'a, Attr> {
    // TODO: Make tests for category error derive
    pub fn transform(self, ctx: &Context) -> Option<Input<'a>> {
        // Validating global_attr whilst it is an enum
        let is_enum = self.variants.is_some();
        if is_enum {
            if self.attr.message.is_some() {
                ctx.spanned_error(
                    &self.original,
                    "#[error(message = ...)] in top-level attribute is not allowed with enums",
                );
                return None;
            }

            if self.attr.subcode.is_some() {
                ctx.spanned_error(
                    &self.original,
                    "#[error(subcode = ...)] in top-level attribute is not allowed with enums",
                );
                return None;
            }
        }

        if !is_enum {
            if self.attr.message.is_none() {
                ctx.spanned_error(
                    &self.original,
                    "#[error(message = ...)] is required",
                );
                return None;
            }

            if self.attr.subcode.is_some() {
                ctx.spanned_error(
                    &self.original,
                    "#[error(subcode = ...)] in top-level attribute is not allowed",
                );
                return None;
            }
        }

        let variants = self.variants.clone().and_then(|v| {
            let mut variants = Vec::new();
            for variant in v {
                variants.push(variant.transform(ctx)?);
            }
            Some(variants)
        });

        Some(Input { variants, global_attr: self.attr.clone(), original: self })
    }
}

impl<'a> base_input::StructGlobal<'a, Attr> {}

impl<'a> base_input::Variant<'a, Attr> {
    pub fn transform(self, ctx: &Context) -> Option<Variant<'a>> {
        let Some(variants) = utils::get_unnamed_variants(&self.original) else {
            ctx.spanned_error(
                &self.original,
                "Every error category must be either in unit or unnamed fields",
            );
            return None;
        };

        let attr = &self.attr;
        let Some(subcode) = attr.subcode else {
            ctx.spanned_error(
                &self.original,
                "#[error(subcode = ...)] is required",
            );
            return None;
        };

        let fields_len = self.original.fields.len();
        if fields_len != 1 && fields_len != 0 {
            ctx.spanned_error(
                &self.original,
                "Every error category must be either unit or newtype variants",
            );
            return None;
        }

        let len_variants = variants.map(|v| v.len()).unwrap_or(0);
        if len_variants == 0 && variants.is_some() {
            ctx.spanned_error(
                &self.original,
                "Every error category has one unnamed field",
            );
            return None;
        }

        if len_variants > 1 {
            ctx.spanned_error(
                &self.original,
                "Every error category has one unnamed field",
            );
            return None;
        }

        if len_variants == 0 && attr.message.is_none() {
            ctx.spanned_error(
                &self.original,
                "#[error(message = ...)] must be used in unit variants",
            );
            return None;
        }

        Some(Variant {
            subcode,
            is_newtype: len_variants == 1,
            message: attr.message.clone(),
            original: self,
        })
    }
}
