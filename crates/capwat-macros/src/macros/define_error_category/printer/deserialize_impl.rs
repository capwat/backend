use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, Token};

use crate::macros::define_error_category::defs::{self, InputCategoryData};

pub struct DeserializeImpl<'a> {
    categories: &'a Punctuated<defs::InputCategory, Token![,]>,
}

impl<'a> DeserializeImpl<'a> {
    pub fn new(input: &'a defs::Input) -> Self {
        Self {
            categories: &input.categories,
        }
    }
}

impl DeserializeImpl<'_> {
    fn render_category_deserializer(
        &self,
        category: &defs::InputCategory,
        tokens: &mut proc_macro2::TokenStream,
    ) {
        let mut other_subcode_deserializers = proc_macro2::TokenStream::new();
        let ident = &category.ident;
        let category_deserializer = syn::Ident::new(&format!("{ident}Deserializer"), ident.span());
        let subcode_ty = syn::Ident::new(&format!("{ident}Subcode"), ident.span());

        match &category.data {
            Some(InputCategoryData::Data { .. }) => {
                todo!()
            }
            Some(InputCategoryData::Subcategories { data, .. }) => {
                for subcategory in data.iter() {
                    let subcategory_ident = &subcategory.ident;
                    if let Some(data) = subcategory.data.as_ref() {
                        let inner_ty = &data.inner;
                        other_subcode_deserializers.extend(quote! {
                            Some(#subcode_ty::#subcategory_ident) => {
                                let value = #inner_ty::deserialize(deserializer)?;
                                Ok(either::Either::Left(#ident::#subcategory_ident(value)))
                            }
                        });
                    } else {
                        other_subcode_deserializers.extend(quote! {
                            Some(#subcode_ty::#subcategory_ident) => {
                                serde::de::IgnoredAny::deserialize(deserializer)?;
                                Ok(either::Either::Left(#ident::#subcategory_ident))
                            }
                        });
                    }
                }
            }
            None => {
                return;
            }
        }

        tokens.extend(quote! {
            struct #category_deserializer<'a>(&'a Option<#subcode_ty>);

            impl<'a, 'de> serde::de::DeserializeSeed<'de> for #category_deserializer<'a> {
                type Value = either::Either<#ident, serde_json::Value>;

                fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    match self.0 {
                        #other_subcode_deserializers
                        Some(#subcode_ty::Other(..)) => {
                            let value = serde_json::Value::deserialize(deserializer)?;
                            Ok(either::Either::Right(value))
                        }
                        // TODO: general variant support
                        None => {
                            let value = serde_json::Value::deserialize(deserializer)?;
                            Ok(either::Either::Right(value))
                        }
                    }
                }
            }
        });
    }

    fn render_category_deserialization(
        &self,
        category: &defs::InputCategory,
        tokens: &mut proc_macro2::TokenStream,
        prereqs: &mut proc_macro2::TokenStream,
    ) {
        let ident = &category.ident;
        let subcode_ty = syn::Ident::new(&format!("{ident}Subcode"), ident.span());
        let category_deserializer = syn::Ident::new(&format!("{ident}Deserializer"), ident.span());

        self.render_category_deserializer(category, prereqs);
        match category.data.as_ref() {
            // TODO: Implement this when it's necessary
            Some(InputCategoryData::Data { .. }) => {
                // tokens.extend(quote!((info) => ));
            }
            Some(InputCategoryData::Subcategories { data, .. }) => {
                let mut catch_tokens = proc_macro2::TokenStream::new();
                for subcategory in data.iter() {
                    let subcategory_ident = &subcategory.ident;
                    if subcategory.data.is_some() {
                        catch_tokens.extend(quote! {
                            Some(#subcode_ty::#subcategory_ident) => {
                                return Err(serde::de::Error::missing_field("data"));
                            }
                        });
                    } else {
                        catch_tokens.extend(quote! {
                            Some(#subcode_ty::#subcategory_ident) => {
                                ErrorCategory::#ident(#ident::#subcategory_ident)
                            }
                        });
                    }
                }

                tokens.extend(quote! {
                    ErrorCodeKind::#ident => {
                        let mut code_exists = false;
                        let mut subcode_exists = false;

                        let mut deserialized: Option<#ident> = None;
                        let mut message: Option<String> = None;
                        let mut data: Option<serde_json::Value> = None;
                        let subcode = self.1.map(#subcode_ty::from_str);

                        while let Some(field) = map.next_key::<Field>()? {
                            match field {
                                Field::Code => {
                                    if code_exists {
                                        return Err(serde::de::Error::duplicate_field("code"));
                                    }
                                    code_exists = true;

                                    let actual_code = map.next_value::<ErrorCodeKind>()?;
                                    if actual_code != code {
                                        return Err(serde::de::Error::custom("unexpected unmatched code what is given from the deserializer"));
                                    }
                                }
                                Field::Subcode => {
                                    if subcode_exists {
                                        return Err(serde::de::Error::duplicate_field("subcode"));
                                    }
                                    subcode_exists = true;

                                    let actual_subcode = map.next_value::<String>()?;
                                    if Some(#subcode_ty::from_str(&actual_subcode))
                                        != subcode
                                    {
                                        return Err(serde::de::Error::custom("unexpected unmatched subcode what is given from the deserializer"));
                                    }
                                }
                                Field::Message => {
                                    if message.is_some() {
                                        return Err(serde::de::Error::duplicate_field("message"));
                                    }
                                    message = Some(map.next_value()?);
                                }
                                Field::Data => {
                                    let deserializer = #category_deserializer(&subcode);
                                    match map.next_value_seed(deserializer)? {
                                        either::Either::Left(that) => {
                                            deserialized = Some(that);
                                        }
                                        either::Either::Right(value) => {
                                            data = Some(value);
                                        }
                                    };
                                }
                                Field::Other => {
                                    map.next_value::<serde::de::IgnoredAny>()?;
                                }
                            }
                        }

                        let category = if let Some(deserialized) = deserialized {
                            ErrorCategory::#ident(deserialized)
                        } else {
                            match subcode {
                                #catch_tokens
                                _ => ErrorCategory::Other(Box::new(OtherError {
                                    code: ErrorCode::#ident(subcode),
                                    data,
                                })),
                            }
                        };

                        Ok(crate::Error { category, message })
                    }
                });
            }
            None => tokens.extend(quote! {
                ErrorCodeKind::#ident => {
                    let mut code_exists = false;
                    let mut subcode_exists = false;

                    let mut message: Option<String> = None;
                    let mut data: Option<serde_json::Value> = None;
                    let subcode = self.1.map(#subcode_ty::from_str);

                    while let Some(field) = map.next_key::<Field>()? {
                        match field {
                            Field::Code => {
                                if code_exists {
                                    return Err(serde::de::Error::duplicate_field("code"));
                                }
                                code_exists = true;

                                let actual_code = map.next_value::<ErrorCodeKind>()?;
                                if actual_code != code {
                                    return Err(serde::de::Error::custom("unexpected unmatched code what is given from the deserializer"));
                                }
                            }
                            Field::Subcode => {
                                if subcode_exists {
                                    return Err(serde::de::Error::duplicate_field("subcode"));
                                }
                                subcode_exists = true;

                                let actual_subcode = map.next_value::<String>()?;
                                if Some(#subcode_ty::from_str(&actual_subcode)) != subcode {
                                    return Err(serde::de::Error::custom("unexpected unmatched subcode what is given from the deserializer"));
                                }
                            }
                            Field::Message => {
                                if message.is_some() {
                                    return Err(serde::de::Error::duplicate_field("message"));
                                }
                                message = Some(map.next_value()?);
                            }
                            Field::Data => {
                                if data.is_some() {
                                    return Err(serde::de::Error::duplicate_field("data"));
                                }
                                data = Some(map.next_value()?);
                            }
                            Field::Other => {
                                map.next_value::<serde::de::IgnoredAny>()?;
                            }
                        }
                    }

                    Ok(crate::Error {
                        category: if subcode.is_none() || data.is_none() {
                            ErrorCategory::#ident
                        } else {
                            ErrorCategory::Other(Box::new(OtherError {
                                code: ErrorCode::#ident(subcode),
                                data,
                            }))
                        },
                        message,
                    })
                }
            })
        }
    }
}

impl ToTokens for DeserializeImpl<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut patterns = proc_macro2::TokenStream::new();
        let mut prereqs = proc_macro2::TokenStream::new();
        for category in self.categories.iter() {
            self.render_category_deserialization(category, &mut patterns, &mut prereqs);
        }

        tokens.extend(quote! {
            #prereqs

            impl<'de> serde::de::DeserializeSeed<'de> for ErrorDeserializer<'_> {
                type Value = crate::Error;

                fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    const FIELDS: &[&str] = &["code", "subcode", "message", "data"];

                    struct Visitor<'a>(Option<&'a str>, Option<&'a str>);

                    #[derive(Debug, Deserialize)]
                    #[serde(field_identifier, rename_all = "lowercase")]
                    enum Field {
                        Code,
                        Subcode,
                        Message,
                        Data,
                        #[serde(other)]
                        Other,
                    }

                    impl<'de> serde::de::Visitor<'de> for Visitor<'_> {
                        type Value = crate::Error;

                        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            f.write_str("Capwat error")
                        }

                        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                        where
                            A: serde::de::MapAccess<'de>,
                        {
                            let raw_code_str = self
                                .0
                                .ok_or_else(|| serde::de::Error::missing_field("code"))?;

                            let code = ErrorCodeKind::from_str(raw_code_str);
                            match code {
                                #patterns
                                ErrorCodeKind::Other(ref code_str) => {
                                    let mut code_exists = false;

                                    let mut subcode: Option<String> = None;
                                    let mut message: Option<String> = None;
                                    let mut data: Option<serde_json::Value> = None;
                                    while let Some(field) = map.next_key::<Field>()? {
                                        match field {
                                            Field::Code => {
                                                if code_exists {
                                                    return Err(serde::de::Error::duplicate_field("code"));
                                                }
                                                code_exists = true;

                                                let actual_code = map.next_value::<ErrorCodeKind>()?;
                                                if actual_code != code {
                                                    return Err(serde::de::Error::custom("unexpected unmatched code what is given from the deserializer"));
                                                }
                                            }
                                            Field::Subcode => {
                                                if subcode.is_some() {
                                                    return Err(serde::de::Error::duplicate_field("subcode"));
                                                }
                                                subcode = Some(map.next_value()?);
                                            }
                                            Field::Message => {
                                                if message.is_some() {
                                                    return Err(serde::de::Error::duplicate_field("message"));
                                                }
                                                message = Some(map.next_value()?);
                                            }
                                            Field::Data => {
                                                if data.is_some() {
                                                    return Err(serde::de::Error::duplicate_field("data"));
                                                }
                                                data = Some(map.next_value()?);
                                            }
                                            Field::Other => {
                                                map.next_value::<serde::de::IgnoredAny>()?;
                                            }
                                        }
                                    }
                                    Ok(crate::Error {
                                        category: ErrorCategory::Other(Box::new(OtherError {
                                            code: ErrorCode::Other {
                                                code: code_str.to_string(),
                                                subcode,
                                            },
                                            data,
                                        })),
                                        message,
                                    })
                                }
                            }
                        }
                    }

                    deserializer.deserialize_struct("Error", FIELDS, Visitor(self.code, self.subcode))
                }
            }
        });
    }
}
