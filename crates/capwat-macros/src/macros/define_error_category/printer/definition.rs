use quote::{quote, ToTokens};
use std::cell::RefCell;
use syn::{punctuated::Punctuated, Token};

use crate::macros::define_error_category::defs::{self, InputCategoryData};

pub struct EnumPrinter<'a> {
    enum_attrs: &'a [syn::Attribute],
    categories: &'a Punctuated<defs::InputCategory, Token![,]>,
}

impl<'a> EnumPrinter<'a> {
    pub fn new(input: &'a defs::Input) -> Self {
        Self {
            enum_attrs: &input.attrs,
            categories: &input.categories,
        }
    }
}

impl ToTokens for EnumPrinter<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut variant_defs = proc_macro2::TokenStream::new();
        let prereqs = RefCell::new(proc_macro2::TokenStream::new());

        for attr in self.enum_attrs {
            attr.to_tokens(tokens);
        }

        for category in self.categories {
            let printer = CategoryPrinter {
                original: category,
                prereqs: &prereqs,
            };
            variant_defs.extend(quote! {
                #printer,
            });
        }

        tokens.extend(quote!(pub enum ErrorCategory {
            #variant_defs
            Other(Box<OtherError>),
        }));

        tokens.extend(prereqs.into_inner());
    }
}

struct CategoryPrinter<'a> {
    original: &'a defs::InputCategory,
    prereqs: &'a RefCell<proc_macro2::TokenStream>,
}

impl quote::ToTokens for CategoryPrinter<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let this = self.original;

        for attr in this.attrs.iter() {
            let mut exclude = false;
            if let Some(segment) = attr.path().segments.first() {
                if segment.ident == "derive" {
                    exclude = true;
                }
            }

            if exclude {
                continue;
            }
            attr.to_tokens(tokens);
        }
        this.ident.to_tokens(tokens);

        match &this.data {
            Some(InputCategoryData::Data { inner, .. }) => {
                tokens.extend(quote!( (#inner) ));
            }
            Some(InputCategoryData::Subcategories { data, .. }) => {
                let printer = SubcategoriesPrinter {
                    category_attrs: &this.attrs,
                    category_ident: &this.ident,
                    data,
                };
                let ident = &this.ident;
                tokens.extend(quote!( (#ident) ));
                printer.to_tokens(&mut self.prereqs.borrow_mut());
            }
            None => {}
        }
    }
}

struct SubcategoriesPrinter<'a> {
    category_attrs: &'a [syn::Attribute],
    category_ident: &'a syn::Ident,
    data: &'a Punctuated<defs::InputSubcategory, Token![,]>,
}

impl quote::ToTokens for SubcategoriesPrinter<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for attr in self.category_attrs.iter() {
            attr.to_tokens(tokens);
        }

        let mut variants_tokens = proc_macro2::TokenStream::new();
        for subcategory in self.data.iter() {
            let ident = &subcategory.ident;
            variants_tokens.extend(quote!(#ident));

            if let Some(data) = subcategory.data.as_ref() {
                let ty = &data.inner;
                variants_tokens.extend(quote!( (#ty) ));
            }

            variants_tokens.extend(quote!(,));
        }

        // Might have to create another struct for it.
        let ident = &self.category_ident;
        tokens.extend(quote! {
            pub enum #ident {
                #variants_tokens
            }
        });
    }
}
