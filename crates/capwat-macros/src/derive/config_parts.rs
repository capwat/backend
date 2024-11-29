use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::ops::Deref;
use syn::{
    parenthesized,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Brace, Comma},
    DeriveInput,
};

pub fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
    let syn::Data::Struct(data) = &input.data else {
        return Err(syn::Error::new(
            input.span(),
            "Enums and unions are not supported for ConfigParts",
        ));
    };

    let fields = Field::get_multiple(data)?;
    let partial_name = syn::Ident::new(&format!("Partial{}", input.ident), input.ident.span());

    let struct_impl = StructImpl::new(&partial_name, &fields, &input)?;
    let trait_impl = TraitImpl { data: &struct_impl };

    Ok(quote! {
        #struct_impl

        #trait_impl
    })
}

struct StructImpl<'a> {
    partial_ident: &'a syn::Ident,
    fields: Punctuated<&'a Field<'a>, Comma>,
    passed_attrs: Vec<syn::Meta>,
    brace: Brace,
    original: &'a syn::DeriveInput,
    impl_trait_manually: bool,
}

impl Deref for StructImpl<'_> {
    type Target = syn::DeriveInput;

    fn deref(&self) -> &Self::Target {
        self.original
    }
}

impl<'a> StructImpl<'a> {
    pub fn new(
        partial_ident: &'a syn::Ident,
        raw_fields: &'a [Field<'a>],
        input: &'a syn::DeriveInput,
    ) -> syn::Result<Self> {
        let mut fields = Punctuated::new();
        let mut passed_attrs = Vec::new();
        let mut impl_trait_manually = false;

        for field in raw_fields {
            fields.push(field);
        }

        for attr in input.attrs.iter() {
            if !attr.path().is_ident("config") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                let path = &meta.path;
                if path.is_ident("attr") {
                    let content;
                    parenthesized!(content in meta.input);

                    let metadata = content.parse::<syn::Meta>()?;
                    passed_attrs.push(metadata);
                } else if path.is_ident("impl_manual") {
                    impl_trait_manually = true;
                } else {
                    let path = path.to_token_stream().to_string().replace(' ', "");
                    return Err(meta.error(format_args!(
                        "unknown ConfigParts struct attribute `{path}`"
                    )));
                }

                Ok(())
            })?;
        }

        Ok(Self {
            partial_ident,
            fields,
            passed_attrs,
            impl_trait_manually,
            brace: Brace::default(),
            original: input,
        })
    }
}

impl ToTokens for StructImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // processing with passed through metas
        for meta in self.passed_attrs.iter() {
            tokens.extend(quote!(#[#meta]));
        }

        let has_non_optional_fields = self.fields.iter().any(|v| match v.override_type.as_ref() {
            Some(n) => !is_wrapped_with_option(n),
            None => false,
        });

        // derive Default i guess?
        if !has_non_optional_fields {
            tokens.extend(quote!(#[derive(Default)]));
        }

        tokens.extend(quote! {
            #[allow(private_interfaces, unused)]
            pub(crate) struct
        });
        self.partial_ident.to_tokens(tokens);
        self.original.generics.to_tokens(tokens);
        self.brace.surround(tokens, |tokens| {
            self.fields.to_tokens(tokens);
        });
    }
}

// TRAIT IMPL //
struct TraitImpl<'a> {
    data: &'a StructImpl<'a>,
}

impl ToTokens for TraitImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.data.impl_trait_manually {
            return;
        }

        let (impl_generics, ty_generics, where_clause) = self.data.generics.split_for_impl();
        let partial_ident = &self.data.partial_ident;

        let mut inner = TokenStream::new();
        for field in self.data.fields.iter() {
            if field.ignore {
                continue;
            }

            let field_name = &field.ident;
            if field.as_struct {
                inner.extend(quote! {
                    self.#field_name = <_ as crate::ConfigParts>::merge(self.#field_name, other.#field_name);
                });
                continue;
            }

            if let Some(override_type) = field.override_type.as_ref()
                && !is_wrapped_with_option(override_type)
            {
                continue;
            }

            inner.extend(quote! {
                self.#field_name = self.#field_name.or(other.#field_name);
            });
        }

        tokens.extend(quote! {
            impl #impl_generics crate::ConfigParts for #partial_ident #ty_generics #where_clause {
                type Output = #partial_ident #ty_generics;

                #[allow(unused_mut)]
                fn merge(mut self, other: Self) -> Self::Output {
                    #inner
                    self
                }
            }
        });
    }
}

// FIELD //
struct Field<'a> {
    ident: syn::Ident,
    override_type: Option<syn::Type>,
    // attributes that needed to passed through the partial struct
    passed_attrs: Vec<syn::Meta>,
    original: &'a syn::Field,
    ignore: bool,
    as_struct: bool,
}

impl<'a> Field<'a> {
    fn get_multiple(data: &'a syn::DataStruct) -> syn::Result<Vec<Self>> {
        if !matches!(data.fields, syn::Fields::Named(..)) {
            return Err(syn::Error::new(
                data.fields.span(),
                "Unnamed and unit fields are not supported for ConfigParts",
            ));
        }

        let mut fields = Vec::new();
        for field in data.fields.iter() {
            fields.push(Self::from_ast(field)?);
        }

        Ok(fields)
    }

    fn from_ast(field: &'a syn::Field) -> syn::Result<Self> {
        let mut passed_attrs = Vec::new();
        let mut override_type = None;
        let mut ignore = false;
        let mut as_struct = false;

        for attr in field.attrs.iter() {
            if !attr.path().is_ident("config") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                let path = &meta.path;
                if path.is_ident("attr") {
                    let content;
                    parenthesized!(content in meta.input);

                    let metadata = content.parse::<syn::Meta>()?;
                    passed_attrs.push(metadata);
                } else if path.is_ident("as_type") {
                    let type_path = parse_lit_into_type("as_type", &meta)?;
                    override_type = Some(type_path);
                } else if path.is_ident("ignore") {
                    ignore = true;
                } else if path.is_ident("as_struct") {
                    as_struct = true;
                } else {
                    let path = path.to_token_stream().to_string().replace(' ', "");
                    return Err(
                        meta.error(format_args!("unknown ConfigParts field attribute `{path}`"))
                    );
                }

                Ok(())
            })?;
        }

        let field = Self {
            ident: field.ident.clone().unwrap(),
            original: field,
            passed_attrs,
            as_struct,
            override_type,
            ignore,
        };

        Ok(field)
    }

    fn get_preferred_type(&self) -> syn::Type {
        if let Some(override_type) = self.override_type.as_ref() {
            override_type.clone()
        } else {
            self.ty.clone()
        }
    }
}

fn is_wrapped_with_option(ty: &syn::Type) -> bool {
    if let syn::Type::Path(inner) = ty {
        if let Some(segment) = inner.path.segments.first() {
            return segment.ident == "Option";
        }
    }
    false
}

impl Deref for Field<'_> {
    type Target = syn::Field;

    fn deref(&self) -> &Self::Target {
        self.original
    }
}

impl ToTokens for Field<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.ignore {
            return;
        }

        // processing with passed through metas
        for meta in self.passed_attrs.iter() {
            tokens.extend(quote!(#[#meta]));
        }

        self.vis.to_tokens(tokens);
        self.ident.to_tokens(tokens);
        tokens.extend(quote!(:));

        let uses_custom_type = self.override_type.is_some();
        let ty = self.get_preferred_type();
        if !uses_custom_type && !is_wrapped_with_option(&ty) {
            tokens.extend(quote!(Option<#ty>));
        } else {
            tokens.extend(quote!(#ty));
        }
    }
}

// stolen from serde. licensed under MIT License
// code: https://github.com/serde-rs/serde/blob/b9dbfcb4ac3b7a663d9efc6eb1387c62302a6fb4/serde_derive/src/internals/attr.rs#L1484-L1504
fn parse_lit_into_type(
    attr_name: &'static str,
    meta: &syn::meta::ParseNestedMeta<'_>,
) -> syn::Result<syn::Type> {
    let string = match get_lit_str(attr_name, meta)? {
        Some(string) => string,
        None => {
            return Err(syn::Error::new(
                Span::call_site(),
                format!("expected {attr_name} to have a type: `{attr_name} = \"...\"`"),
            ))
        }
    };

    match string.parse() {
        Ok(expr) => Ok(expr),
        Err(_) => Err(syn::Error::new(
            Span::call_site(),
            format!("failed to parse type: {:?}", string.value()),
        )),
    }
}

// code: https://github.com/serde-rs/serde/blob/master/serde_derive/src/internals/attr.rs#L1426-L1460
fn get_lit_str(
    attr_name: &'static str,
    meta: &syn::meta::ParseNestedMeta<'_>,
) -> syn::Result<Option<syn::LitStr>> {
    let expr: syn::Expr = meta.value()?.parse()?;
    let mut value = &expr;
    while let syn::Expr::Group(e) = value {
        value = &e.expr;
    }
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(lit),
        ..
    }) = value
    {
        let suffix = lit.suffix();
        if !suffix.is_empty() {
            return Err(syn::Error::new(
                Span::call_site(),
                format!("unexpected suffix `{suffix}` on string literal"),
            ));
        }
        Ok(Some(lit.clone()))
    } else {
        Err(syn::Error::new(
            expr.span(),
            format!("expected {attr_name} to be a string: `{attr_name} = \"...\"`",),
        ))
    }
}
