use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{meta::ParseNestedMeta, Token};

use crate::legacy::internals::{
  utils::{get_lit_int, get_lit_str, parse_lit_into_path},
  Context,
};

pub const PATH_OPTIONAL: &str = "optional";
pub const PATH_LENGTH: &str = "length";
pub const PATH_NESTED: &str = "nested";
pub const PATH_RANGES: &str = "ranges";
pub const PATH_WITH: &str = "with";
pub const PATH_ERROR: &str = "error";

pub const PATH_LENGTH_MAX: &str = "max";
pub const PATH_LENGTH_MIN: &str = "min";
pub const PATH_LENGTH_EQUAL: &str = "equal";

pub const VALIDATE: &str = "validate";

struct Attribute<'a, T> {
  ctx: &'a Context,
  name: &'static str,
  tokens: TokenStream,
  value: Option<T>,
}

impl<'a, T> Attribute<'a, T> {
  fn new(ctx: &'a Context, name: &'static str) -> Self {
    Self {
      ctx,
      name,
      tokens: TokenStream::new(),
      value: None,
    }
  }

  fn set<A: ToTokens>(&mut self, obj: A, value: T) {
    let tokens = obj.into_token_stream();

    if self.value.is_some() {
      let msg = format!("duplicated validator attribute `{}`", self.name);
      self.ctx.spanned_error(tokens, msg);
    } else {
      self.tokens = tokens;
      self.value = Some(value);
    }
  }

  // fn set_opt<A: ToTokens>(&mut self, obj: A, value: Option<T>) {
  //   if let Some(value) = value {
  //     self.set(obj, value);
  //   }
  // }

  // fn set_if_none(&mut self, value: T) {
  //   if self.value.is_none() {
  //     self.value = Some(value);
  //   }
  // }

  fn get(self) -> Option<T> {
    self.value
  }

  // fn get_with_tokens(self) -> Option<(TokenStream, T)> {
  //   match self.value {
  //     Some(v) => Some((self.tokens, v)),
  //     None => None,
  //   }
  // }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Ranges {
  pub min: Option<isize>,
  pub max: Option<isize>,
}

impl Ranges {
  fn from_parsed_meta(ctx: &Context, meta: &ParseNestedMeta<'_>) -> syn::Result<Self> {
    // #[validate(length([min = ...] [max = ...] [equal = ...]))]
    let mut field = Self::default();
    meta.parse_nested_meta(|meta| {
      let meta_path = &meta.path;
      meta.input.parse::<Token![=]>()?;
      if meta_path.is_ident(PATH_LENGTH_MIN) {
        if let Some(value) = get_lit_int::<isize>(ctx, PATH_LENGTH, PATH_LENGTH_MIN, &meta)? {
          field.min = Some(value);
        }
      } else if meta_path.is_ident(PATH_LENGTH_MAX) {
        if let Some(value) = get_lit_int::<isize>(ctx, PATH_LENGTH, PATH_LENGTH_MAX, &meta)? {
          field.max = Some(value);
        }
      } else {
        return Err(meta.error(format_args!(
          "invalid attribute for {0}, expected `{0}({1} = ..., {2} = ...)`",
          PATH_RANGES, PATH_LENGTH_MIN, PATH_LENGTH_MAX,
        )));
      }
      Ok(())
    })?;

    Ok(field)
  }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Length {
  pub min: Option<usize>,
  pub max: Option<usize>,
  pub equal: Option<usize>,
}

impl Length {
  fn from_parsed_meta(ctx: &Context, meta: &ParseNestedMeta<'_>) -> syn::Result<Self> {
    // #[validate(length([min = ...] [max = ...] [equal = ...]))]
    let mut field = Self::default();
    meta.parse_nested_meta(|meta| {
      let meta_path = &meta.path;
      meta.input.parse::<Token![=]>()?;
      if meta_path.is_ident(PATH_LENGTH_MIN) {
        if let Some(value) = get_lit_int::<usize>(ctx, PATH_LENGTH, PATH_LENGTH_MIN, &meta)? {
          field.min = Some(value);
        }
      } else if meta_path.is_ident(PATH_LENGTH_MAX) {
        if let Some(value) = get_lit_int::<usize>(ctx, PATH_LENGTH, PATH_LENGTH_MAX, &meta)? {
          field.max = Some(value);
        }
      } else if meta_path.is_ident(PATH_LENGTH_EQUAL) {
        if let Some(value) = get_lit_int::<usize>(ctx, PATH_LENGTH, PATH_LENGTH_EQUAL, &meta)? {
          field.equal = Some(value);
        }
      } else {
        return Err(meta.error(format_args!(
          "invalid attribute for {0}, expected `{0}({1} = ..., {2} = ...)` or `{3}(equal = ...)`",
          PATH_LENGTH, PATH_LENGTH_MIN, PATH_LENGTH_MAX, PATH_LENGTH_EQUAL
        )));
      }
      Ok(())
    })?;

    Ok(field)
  }
}

pub struct Field {
  // #[validate(error = "Invalid username")]
  custom_error_msg: Option<syn::LitStr>,
  // #[validate(length(min = 1, max = 15))]
  // #[validate(length(equal = 5))]
  length: Option<Length>,
  // #[validate(ranges(min = -1, max = 15))]
  ranges: Option<Ranges>,
  // #[validate(with = "validate_username")]
  checker: Option<syn::ExprPath>,
  // It allows to validate the inner fields of a type
  // #[validate(nested)]
  nested: Option<()>,
  // Accepts `Option` type
  optional: Option<()>,
}

impl Field {
  pub fn requires_extend(&self) -> bool {
    self.allow_nested() || self.length.is_some() || self.ranges.is_some() || self.checker.is_some()
  }

  pub fn allow_optional(&self) -> bool {
    self.optional.is_some()
  }

  pub fn allow_nested(&self) -> bool {
    self.nested.is_some()
  }

  pub fn custom_error_msg(&self) -> Option<&syn::LitStr> {
    self.custom_error_msg.as_ref()
  }

  pub fn length(&self) -> Option<&Length> {
    self.length.as_ref()
  }

  pub fn ranges(&self) -> Option<&Ranges> {
    self.ranges.as_ref()
  }

  pub fn with(&self) -> Option<&syn::ExprPath> {
    self.checker.as_ref()
  }
}

impl Field {
  pub fn from_ast(ctx: &Context, field: &syn::Field) -> Self {
    let mut error_attr = Attribute::<syn::LitStr>::new(ctx, PATH_ERROR);
    let mut length_attr = Attribute::<Length>::new(ctx, PATH_LENGTH);
    let mut nested_attr = Attribute::<()>::new(ctx, PATH_NESTED);
    let mut ranges_attr = Attribute::<Ranges>::new(ctx, PATH_RANGES);
    let mut with_attr = Attribute::<syn::ExprPath>::new(ctx, PATH_WITH);
    let mut optional_attr = Attribute::<()>::new(ctx, PATH_NESTED);

    for attr in &field.attrs {
      if !attr.path().is_ident(VALIDATE) {
        continue;
      }

      if let syn::Meta::List(meta) = &attr.meta {
        if meta.tokens.is_empty() {
          continue;
        }
      }

      if let Err(err) = attr.parse_nested_meta(|meta| {
        let meta_path = &meta.path;
        if meta_path.is_ident(PATH_WITH) {
          if let Some(path) = parse_lit_into_path(ctx, PATH_WITH, &meta)? {
            with_attr.set(meta_path, path);
          }
        } else if meta_path.is_ident(PATH_LENGTH) {
          let field = Length::from_parsed_meta(ctx, &meta)?;
          length_attr.set(meta_path, field);
        } else if meta_path.is_ident(PATH_RANGES) {
          let field = Ranges::from_parsed_meta(ctx, &meta)?;
          ranges_attr.set(meta_path, field);
        } else if meta_path.is_ident(PATH_NESTED) {
          nested_attr.set(meta_path, ());
        } else if meta_path.is_ident(PATH_ERROR) {
          if let Some(value) = get_lit_str(ctx, PATH_ERROR, &meta)? {
            error_attr.set(meta_path, value);
          }
        } else if meta_path.is_ident(PATH_OPTIONAL) {
          optional_attr.set(meta_path, ());
        } else {
          let path = meta_path.to_token_stream().to_string().replace(' ', "");
          return Err(meta.error(format_args!("unknown validator field attribute `{path}`")));
        }
        Ok(())
      }) {
        ctx.error(err);
      }
    }

    Self {
      custom_error_msg: error_attr.get(),
      length: length_attr.get(),
      nested: nested_attr.get(),
      ranges: ranges_attr.get(),
      checker: with_attr.get(),
      optional: optional_attr.get(),
    }
  }
}
