use crate::utils::Context;

pub struct Input<'a, A: GlobalAttrParse + VariantAttrParse> {
    pub ident: syn::Ident,
    pub variants: Option<Vec<Variant<'a, A>>>,
    pub generics: &'a syn::Generics,
    pub original: &'a syn::DeriveInput,
    pub attr: A,
}

pub struct EnumInput<'a, A: VariantAttrParse> {
    pub ident: syn::Ident,
    pub variants: Vec<Variant<'a, A>>,
    pub generics: &'a syn::Generics,
    pub original: &'a syn::DeriveInput,
}

#[derive(Clone)]
pub struct StructGlobal<'a, A: GlobalAttrParse> {
    pub member: syn::Member,
    pub attrs: A,
    pub original: &'a syn::Field,
}

#[derive(Clone)]
pub struct Variant<'a, A: VariantAttrParse> {
    pub ident: syn::Ident,
    pub style: Style,
    pub attr: A,
    pub original: &'a syn::Variant,
}

#[derive(Copy, Clone)]
pub enum Style {
    /// Named fields.
    Struct,
    /// Many unnamed fields.
    Tuple,
    /// No fields inside
    Unit,
}

impl<'a, A: GlobalAttrParse + VariantAttrParse> Input<'a, A> {
    pub fn from_derive(
        ctx: &Context,
        input: &'a syn::DeriveInput,
    ) -> Option<Self> {
        Some(Self {
            ident: input.ident.clone(),
            variants: match &input.data {
                syn::Data::Struct(..) => None,
                syn::Data::Enum(data) => {
                    let mut variants = Vec::new();
                    for variant in &data.variants {
                        variants.push(Self::from_variant_ast(ctx, variant)?);
                    }
                    Some(variants)
                },
                _ => {
                    ctx.spanned_error(
                        input,
                        "this macro does not support derive for unions",
                    );
                    None
                },
            },
            generics: &input.generics,
            original: input,
            attr: <A as GlobalAttrParse>::from_ast(ctx, input),
        })
    }

    fn from_variant_ast(
        ctx: &Context,
        variant: &'a syn::Variant,
    ) -> Option<Variant<'a, A>> {
        if matches!(variant.fields, syn::Fields::Named(..)) {
            ctx.spanned_error(
                variant,
                "this macro does not support variants with named fields",
            );
            return None;
        }

        // let (style, fields) = Self::from_fields(ctx, &variant.fields);
        Some(Variant {
            ident: variant.ident.clone(),
            style: if matches!(variant.fields, syn::Fields::Unit) {
                Style::Unit
            } else {
                Style::Tuple
            },
            attr: <A as VariantAttrParse>::from_ast(ctx, variant),
            original: variant,
        })
    }
}

impl<'a, A: VariantAttrParse> EnumInput<'a, A> {
    pub fn from_derive(
        ctx: &Context,
        input: &'a syn::DeriveInput,
    ) -> Option<Self> {
        Some(Self {
            ident: input.ident.clone(),
            variants: Variant::from_derive(ctx, input)?,
            generics: &input.generics,
            original: input,
        })
    }
}

impl<'a, A: VariantAttrParse> Variant<'a, A> {
    fn from_derive(
        ctx: &Context,
        input: &'a syn::DeriveInput,
    ) -> Option<Vec<Self>> {
        match &input.data {
            syn::Data::Enum(data) => {
                let mut variants = Vec::new();
                for variant in &data.variants {
                    variants.push(Self::from_ast(ctx, variant)?);
                }
                Some(variants)
            },
            _ => {
                ctx.spanned_error(
                    input,
                    "this macro does not support derive for structs and unions",
                );
                None
            },
        }
    }

    fn from_ast(ctx: &Context, variant: &'a syn::Variant) -> Option<Self> {
        if matches!(variant.fields, syn::Fields::Named(..)) {
            ctx.spanned_error(
                variant,
                "this macro does not support variants with named fields",
            );
            return None;
        }

        // let (style, fields) = Self::from_fields(ctx, &variant.fields);
        Some(Variant {
            ident: variant.ident.clone(),
            style: if matches!(variant.fields, syn::Fields::Unit) {
                Style::Unit
            } else {
                Style::Tuple
            },
            attr: A::from_ast(ctx, variant),
            original: variant,
        })
    }
}

pub trait GlobalAttrParse {
    fn from_ast(ctx: &Context, data: &syn::DeriveInput) -> Self;
}

pub trait VariantAttrParse {
    fn from_ast(ctx: &Context, variant: &syn::Variant) -> Self;
}

impl GlobalAttrParse for () {
    fn from_ast(_ctx: &Context, _variant: &syn::DeriveInput) -> Self {}
}

impl VariantAttrParse for () {
    fn from_ast(_ctx: &Context, _variant: &syn::Variant) -> Self {}
}
