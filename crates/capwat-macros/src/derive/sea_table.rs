use heck::ToPascalCase;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, DeriveInput};

use crate::utils::{get_lit_str, parse_lit_into_type};

pub fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
    let syn::Data::Struct(data) = &input.data else {
        return Err(syn::Error::new(
            input.span(),
            "Enums and unions are not supported for SeaTable",
        ));
    };

    // Generating Ident enum
    let fields = read_fields(data)?;
    let global_config = read_global_config(&input)?;
    let (_, ident_enum_tokens) = generate_ident_enum(&global_config, &input, data)?;

    let changeset_impl = generate_changeset_impl(
        &input,
        global_config.changeset_type.as_ref(),
        &global_config.table_name,
        &fields,
    );

    Ok(quote! {
        #ident_enum_tokens
        #changeset_impl
    })
}

fn generate_changeset_impl(
    input: &DeriveInput,
    ty: Option<&syn::Type>,
    table_name: &str,
    fields: &[Field],
) -> TokenStream {
    let Some(ty) = ty else {
        return TokenStream::new();
    };

    let mut modifiers = Vec::new();
    let table_name = syn::Ident::new(&table_name.to_pascal_case(), Span::call_site());
    let ident_name = syn::Ident::new(&format!("{}Ident", input.ident), input.span());

    for field in fields {
        if field.exclude_in_changeset {
            continue;
        }

        let variant = syn::Ident::new(
            &field.ident.to_string().to_pascal_case(),
            field.ident.span(),
        );
        let ident = &field.ident;
        modifiers.push(quote! {
            if let Some(value) = self.#ident {
                values.push((#ident_name::#variant, value.into()));
            }
        });
    }

    quote! {
        impl #ty {
            pub(crate) fn make_changeset_sql(&self, stmt: &mut sea_query::UpdateStatement) {
                let mut values: Vec<(#ident_name, sea_query::SimpleExpr)> = Vec::new();
                #(#modifiers)*
                stmt.table(#ident_name::#table_name).values(values).returning_all();
            }
        }
    }
}

struct Field {
    ident: syn::Ident,
    exclude_in_changeset: bool,
}

fn read_fields(data: &syn::DataStruct) -> syn::Result<Vec<Field>> {
    let mut fields = Vec::new();
    for field in &data.fields {
        let ident = field.ident.as_ref().ok_or_else(|| {
            syn::Error::new(field.span(), "Unnamed field is not allowed for SeaTable")
        })?;

        let name = ident.to_string();
        let mut should_exclude = name == "id" || name == "created" || name == "updated";
        for attr in field.attrs.iter() {
            if !attr.path().is_ident("sea_table") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                let path = &meta.path;
                if path.is_ident("exclude_in_changeset") {
                    should_exclude = true;
                } else {
                    let path = path.to_token_stream().to_string().replace(' ', "");
                    return Err(
                        meta.error(format_args!("unknown SeaTable field attribute `{path}`"))
                    );
                }
                Ok(())
            })?;
        }

        fields.push(Field {
            ident: ident.clone(),
            exclude_in_changeset: should_exclude,
        });
    }
    Ok(fields)
}

struct GlobalConfig {
    changeset_type: Option<syn::Type>,
    table_name: String,
}

fn read_global_config(input: &DeriveInput) -> syn::Result<GlobalConfig> {
    let mut table_name: Option<String> = None;
    let mut changeset_type: Option<syn::Type> = None;
    for attr in input.attrs.iter() {
        if !attr.path().is_ident("sea_table") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            let path = &meta.path;
            if path.is_ident("table_name") {
                table_name = Some(get_lit_str("table_name", &meta)?.value());
            } else if path.is_ident("changeset") {
                changeset_type = Some(parse_lit_into_type("changeset", &meta)?);
            } else {
                let path = path.to_token_stream().to_string().replace(' ', "");
                return Err(meta.error(format_args!("unknown SeaTable global attribute `{path}`")));
            }
            Ok(())
        })?;
    }

    Ok(GlobalConfig {
        changeset_type,
        table_name: table_name
            .ok_or_else(|| syn::Error::new(input.span(), "Missing #[sea(table_name = ...)]"))?,
    })
}

fn generate_ident_enum(
    global_config: &GlobalConfig,
    input: &DeriveInput,
    data: &syn::DataStruct,
) -> syn::Result<(syn::Ident, TokenStream)> {
    let mut fields = Vec::<syn::Ident>::new();
    fields.push(syn::Ident::new(
        &global_config.table_name.to_pascal_case(),
        input.span(),
    ));

    for field in &data.fields {
        let that = field.ident.as_ref().ok_or_else(|| {
            syn::Error::new(field.span(), "Unnamed field is not allowed for SeaTable")
        })?;

        fields.push(syn::Ident::new(
            &that.to_string().to_pascal_case(),
            field.span(),
        ));
    }

    let ident_name = syn::Ident::new(&format!("{}Ident", input.ident), input.span());
    let tokens = quote! {
        #[allow(unused)]
        #[derive(sea_query::Iden)]
        pub(crate) enum #ident_name {
            #(#fields,)*
        }
    };

    Ok((ident_name, tokens))
}
