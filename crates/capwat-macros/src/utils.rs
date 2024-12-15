use proc_macro2::Span;
use syn::spanned::Spanned;

// stolen from serde. licensed under MIT License
// code: https://github.com/serde-rs/serde/blob/b9dbfcb4ac3b7a663d9efc6eb1387c62302a6fb4/serde_derive/src/internals/attr.rs#L1484-L1504
pub fn parse_lit_into_type(
    attr_name: &'static str,
    meta: &syn::meta::ParseNestedMeta<'_>,
) -> syn::Result<syn::Type> {
    let string = get_lit_str(attr_name, meta)?;

    match string.parse() {
        Ok(expr) => Ok(expr),
        Err(_) => Err(syn::Error::new(
            Span::call_site(),
            format!("failed to parse type: {:?}", string.value()),
        )),
    }
}

// code: https://github.com/serde-rs/serde/blob/master/serde_derive/src/internals/attr.rs#L1426-L1460
pub fn get_lit_str(
    attr_name: &'static str,
    meta: &syn::meta::ParseNestedMeta<'_>,
) -> syn::Result<syn::LitStr> {
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
        Ok(lit.clone())
    } else {
        Err(syn::Error::new(
            expr.span(),
            format!("expected {attr_name} to be a string: `{attr_name} = \"...\"`",),
        ))
    }
}
