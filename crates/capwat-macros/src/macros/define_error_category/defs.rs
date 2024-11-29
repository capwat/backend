//! **Syntax**:
//! ```txt,no-run
//! [ <attr> ]*
//! pub enum <ident> {
//!     [
//!         [ <attr> ]*
//!         <category> [ <subcategory>, ]+ | ( <data> )
//!     ]*
//! }
//! ```

use syn::{braced, parenthesized, punctuated::Punctuated, Token};

pub struct Input {
    pub attrs: Vec<syn::Attribute>,
    pub categories: Punctuated<InputCategory, Token![,]>,
}

impl syn::parse::Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let content;
        let _brace = braced!(content in input);
        Ok(Self {
            attrs,
            categories: content.parse_terminated(InputCategory::parse, Token![,])?,
        })
    }
}

pub struct InputCategory {
    pub attrs: Vec<syn::Attribute>,
    pub ident: syn::Ident,
    pub data: Option<InputCategoryData>,
}

pub enum InputCategoryData {
    Subcategories {
        #[allow(unused)]
        brace_token: syn::token::Brace,
        data: Punctuated<InputSubcategory, Token![,]>,
    },
    Data {
        #[allow(unused)]
        paren_token: syn::token::Paren,
        inner: syn::Type,
    },
}

impl syn::parse::Parse for InputCategory {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let ident = input.parse::<syn::Ident>()?;
        let data = if input.peek(syn::token::Brace) {
            let content;
            let brace_token = braced!(content in input);
            Some(InputCategoryData::Subcategories {
                brace_token,
                data: content.parse_terminated(InputSubcategory::parse, Token![,])?,
            })
        } else if input.peek(syn::token::Paren) {
            let content;
            let paren_token = parenthesized!(content in input);
            Some(InputCategoryData::Data {
                paren_token,
                inner: content.parse()?,
            })
        } else {
            None
        };

        Ok(Self { attrs, ident, data })
    }
}

pub struct InputSubcategory {
    pub attrs: Vec<syn::Attribute>,
    pub ident: syn::Ident,
    pub data: Option<InputSubcategoryData>,
}

pub struct InputSubcategoryData {
    #[allow(unused)]
    pub paren_token: syn::token::Paren,
    pub inner: syn::Type,
}

impl syn::parse::Parse for InputSubcategory {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let ident = input.parse::<syn::Ident>()?;
        let data = if input.peek(syn::token::Paren) {
            let content;
            let paren_token = parenthesized!(content in input);
            let inner = content.parse::<syn::Type>()?;
            Some(InputSubcategoryData { paren_token, inner })
        } else {
            None
        };

        Ok(Self { attrs, ident, data })
    }
}
