use super::Context;
use proc_macro2::Span;
use std::{fmt::Display, str::FromStr};
use syn::meta::ParseNestedMeta;

pub fn make_lit_str(value: impl Display) -> syn::LitStr {
  syn::LitStr::new(&value.to_string(), Span::call_site())
}

// pub fn unraw(ident: &Ident) -> String {
//   ident.to_string().trim_start_matches("r#").to_owned()
// }

pub fn get_lit_int<T: FromStr>(
  ctx: &Context,
  attr_name: &'static str,
  meta_item_name: &'static str,
  meta: &ParseNestedMeta<'_>,
) -> syn::Result<Option<T>>
where
  <T as FromStr>::Err: std::fmt::Display,
{
  let value = meta.input.parse::<syn::LitInt>()?;
  Ok(match value.base10_parse() {
    Ok(value) => Some(value),
    Err(_) => {
      ctx.spanned_error(
        &value,
        format!(
          "expected validator {} attribute to be a valid integer: `{} = \"...\"`",
          attr_name, meta_item_name
        ),
      );
      None
    }
  })
}

pub fn parse_lit_into_path(
  ctx: &Context,
  attr_name: &'static str,
  meta: &ParseNestedMeta<'_>,
) -> syn::Result<Option<syn::ExprPath>> {
  let string = match get_lit_str(ctx, attr_name, meta)? {
    Some(string) => string,
    None => return Ok(None),
  };

  Ok(match string.parse() {
    Ok(path) => Some(path),
    Err(_) => {
      ctx.spanned_error(
        &string,
        format!("failed to parse path: {:?}", string.value()),
      );
      None
    }
  })
}

pub fn get_lit_str(
  ctx: &Context,
  attr_name: &'static str,
  meta: &ParseNestedMeta<'_>,
) -> syn::Result<Option<syn::LitStr>> {
  get_lit_str2(ctx, attr_name, attr_name, meta)
}

pub fn get_lit_str2(
  ctx: &Context,
  attr_name: &'static str,
  meta_item_name: &'static str,
  meta: &ParseNestedMeta<'_>,
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
      ctx.spanned_error(
        lit,
        format!("unexpected suffix `{}` on string literal", suffix),
      );
    }
    Ok(Some(lit.clone()))
  } else {
    ctx.spanned_error(
      expr,
      format!(
        "expected validator {} attribute to be a string: `{} = \"...\"`",
        attr_name, meta_item_name
      ),
    );
    Ok(None)
  }
}
