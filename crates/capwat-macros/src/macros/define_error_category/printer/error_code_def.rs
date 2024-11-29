use heck::ToSnekCase;
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, Token};

use crate::macros::define_error_category::defs;

pub struct ErrorCodePrinter<'a> {
    categories: &'a Punctuated<defs::InputCategory, Token![,]>,
}

impl<'a> ErrorCodePrinter<'a> {
    pub fn new(input: &'a defs::Input) -> Self {
        Self {
            categories: &input.categories,
        }
    }
}

impl ToTokens for ErrorCodePrinter<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut inner_tokens = proc_macro2::TokenStream::new();
        let mut inner_raw_tokens = proc_macro2::TokenStream::new();
        let mut inner_raw_de_tokens = proc_macro2::TokenStream::new();
        let mut from_str_impl_tokens = proc_macro2::TokenStream::new();

        let mut prereqs = proc_macro2::TokenStream::new();
        for category in self.categories {
            let mut category_enum_from_str_tokens = proc_macro2::TokenStream::new();

            let ident = &category.ident;
            inner_raw_tokens.extend(quote!(#ident));

            let code = ident.to_string().to_snek_case();
            inner_raw_de_tokens.extend(quote!(#code => Ok(ErrorCodeKind::#ident),));
            from_str_impl_tokens.extend(quote!( #code => ErrorCodeKind::#ident, ));

            let mut category_enum_tokens = proc_macro2::TokenStream::new();
            let category_enum_ident =
                syn::Ident::new(&format!("{ident}Subcode"), proc_macro2::Span::call_site());

            if let Some(defs::InputCategoryData::Subcategories {
                data: subcategories,
                ..
            }) = category.data.as_ref()
            {
                for subcategory in subcategories.iter() {
                    for attr in subcategory.attrs.iter() {
                        attr.to_tokens(&mut category_enum_tokens);
                    }

                    let ident = &subcategory.ident;
                    category_enum_tokens.extend(quote! {
                        #ident,
                    });

                    let subcategory_code = ident.to_string().to_snek_case();
                    category_enum_from_str_tokens.extend(quote! {
                        #subcategory_code => #category_enum_ident::#ident,
                    });
                }
            }
            inner_tokens.extend(quote!( #ident ( Option<#category_enum_ident> ), ));
            prereqs.extend(quote! {
                #[derive(Debug, Clone, PartialEq, Eq, Hash)]
                pub enum #category_enum_ident {
                    #category_enum_tokens
                    Other(String),
                }

                impl #category_enum_ident {
                    #[must_use]
                    pub fn from_str(s: &str) -> Self {
                        match s {
                            #category_enum_from_str_tokens
                            _ => Self::Other(s.to_string()),
                        }
                    }
                }
            });

            inner_raw_tokens.extend(quote!(,));
        }

        tokens.extend(quote! {
            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
            pub enum ErrorCode {
                #inner_tokens
                Other {
                    code: String,
                    subcode: Option<String>,
                }
            }

            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
            pub enum ErrorCodeKind {
                #inner_raw_tokens
                Other(String),
            }

            impl ErrorCodeKind {
                #[must_use]
                pub fn from_str(s: &str) -> Self {
                    match s {
                        #from_str_impl_tokens
                        _ => Self::Other(s.to_string()),
                    }
                }
            }

            impl<'de> ::serde::Deserialize<'de> for ErrorCodeKind {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    struct Visitor;

                    impl<'de> serde::de::Visitor<'de> for Visitor {
                        type Value = ErrorCodeKind;

                        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            f.write_str("capwat error code")
                        }

                        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                        where
                            E: ::serde::de::Error,
                        {
                            match v {
                                #inner_raw_de_tokens
                                _ => Ok(ErrorCodeKind::Other(v.to_string())),
                            }
                        }
                    }

                    deserializer.deserialize_str(Visitor)
                }
            }
        });
        tokens.extend(prereqs);
    }
}
