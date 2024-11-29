use heck::ToSnekCase;
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, Token};

use crate::macros::define_error_category::defs::{self, InputCategoryData};

pub struct SerializeImpl<'a> {
    categories: &'a Punctuated<defs::InputCategory, Token![,]>,
}

impl<'a> SerializeImpl<'a> {
    pub fn new(input: &'a defs::Input) -> Self {
        Self {
            categories: &input.categories,
        }
    }
}

impl SerializeImpl<'_> {
    fn render_category_serialization(
        &self,
        category: &defs::InputCategory,
        tokens: &mut proc_macro2::TokenStream,
        other_patterns: &mut proc_macro2::TokenStream,
    ) {
        let ident = &category.ident;
        let category_code = ident.to_string().to_snek_case();
        let subcode_enum_ident = syn::Ident::new(&format!("{ident}Subcode"), category.ident.span());

        let mut inner_other_patterns = proc_macro2::TokenStream::new();
        match category.data.as_ref() {
            // TODO: Implement this when it's necessary
            Some(InputCategoryData::Data { .. }) => {
                // tokens.extend(quote!((info) => ));
            }
            Some(InputCategoryData::Subcategories { data, .. }) => {
                let mut subcategory_tokens = proc_macro2::TokenStream::new();

                for subcategory in data.iter() {
                    let subcategory_ident = &subcategory.ident;
                    let subcode = subcategory.ident.to_string().to_snek_case();

                    inner_other_patterns.extend(quote! {
                        Some(#subcode_enum_ident::#subcategory_ident) => {
                            let mut map = serializer.serialize_map(Some(
                                2 + if self.message.is_some() { 1 } else { 0 }
                                    + if other.data.is_some() { 1 } else { 0 },
                            ))?;

                            map.serialize_entry("code", #category_code)?;
                            map.serialize_entry("subcode", #subcode)?;
                            if let Some(message) = self.message.as_ref() {
                                map.serialize_entry("message", message)?;
                            }

                            if let Some(data) = other.data.as_ref() {
                                map.serialize_entry("data", data)?;
                            }
                            map.end()
                        }
                    });
                    subcategory_tokens.extend(quote!( #ident::#subcategory_ident ));

                    if subcategory.data.is_some() {
                        subcategory_tokens.extend(quote!( (data) => {
                            let mut map = serializer.serialize_map(Some(3 + if self.message.is_some() { 1 } else { 0 }))?;
                            map.serialize_entry("code", #category_code)?;
                            map.serialize_entry("subcode", #subcode)?;
                            if let Some(message) = self.message.as_ref() {
                                map.serialize_entry("message", message)?;
                            }
                            map.serialize_entry("data", data)?;
                            map.end()
                        } ));
                    } else {
                        subcategory_tokens.extend(quote!( => {
                            let mut map = serializer.serialize_map(Some(2 + if self.message.is_some() { 1 } else { 0 }))?;
                            map.serialize_entry("code", #category_code)?;
                            map.serialize_entry("subcode", #subcode)?;
                            if let Some(message) = self.message.as_ref() {
                                map.serialize_entry("message", message)?;
                            }
                            map.end()
                        } ));
                    }
                }
                tokens.extend(quote! {
                    ErrorCategory::#ident (subcategory) => match subcategory {
                        #subcategory_tokens
                    }
                });
            }
            None => {
                tokens.extend(quote!( ErrorCategory::#ident => {
                    let mut map = serializer.serialize_map(Some(1 + if self.message.is_some() { 1 } else { 0 }))?;
                    map.serialize_entry("code", #category_code)?;
                    if let Some(message) = self.message.as_ref() {
                        map.serialize_entry("message", message)?;
                    }
                    map.end()
                }));
            }
        };

        other_patterns.extend(quote! {
            ErrorCode::#ident(subcode) => match subcode {
                #inner_other_patterns
                Some(#subcode_enum_ident::Other(subcode)) => {
                    let mut map = serializer.serialize_map(Some(
                        2 + if self.message.is_some() { 1 } else { 0 }
                            + if other.data.is_some() { 1 } else { 0 },
                    ))?;

                    map.serialize_entry("code", "register_failed")?;
                    map.serialize_entry("subcode", subcode)?;
                    if let Some(message) = self.message.as_ref() {
                        map.serialize_entry("message", message)?;
                    }

                    if let Some(data) = other.data.as_ref() {
                        map.serialize_entry("data", data)?;
                    }
                    map.end()
                },
                None => {
                    let mut map = serializer.serialize_map(Some(
                        1 + if self.message.is_some() { 1 } else { 0 }
                            + if other.data.is_some() { 1 } else { 0 },
                    ))?;
                    map.serialize_entry("code", #category_code)?;
                    if let Some(message) = self.message.as_ref() {
                        map.serialize_entry("message", message)?;
                    }
                    if let Some(data) = other.data.as_ref() {
                        map.serialize_entry("data", data)?;
                    }
                    map.end()
                }
            }
        });
    }
}

impl ToTokens for SerializeImpl<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut patterns = proc_macro2::TokenStream::new();
        let mut other_patterns = proc_macro2::TokenStream::new();
        for category in self.categories.iter() {
            self.render_category_serialization(category, &mut patterns, &mut other_patterns);
        }

        tokens.extend(quote! {
            impl ::serde::Serialize for crate::Error {
                fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
                where
                    S: ::serde::Serializer,
                {
                    use ::serde::ser::SerializeMap;
                    match &self.category {
                        #patterns
                        ErrorCategory::Other(other) => match &other.code {
                            #other_patterns
                            ErrorCode::Other { code, subcode } => {
                                let mut map = serializer.serialize_map(Some(
                                    1 + if self.message.is_some() { 1 } else { 0 }
                                        + if subcode.is_some() { 1 } else { 0 }
                                        + if other.data.is_some() { 1 } else { 0 },
                                ))?;

                                map.serialize_entry("code", code)?;
                                if let Some(subcode) = subcode.as_ref() {
                                    map.serialize_entry("subcode", subcode)?;
                                }

                                if let Some(message) = self.message.as_ref() {
                                    map.serialize_entry("message", message)?;
                                }

                                if let Some(data) = other.data.as_ref() {
                                    map.serialize_entry("data", data)?;
                                }
                                map.end()
                            }
                            _ => todo!()
                        }
                    }
                }
            }
        });
    }
}
