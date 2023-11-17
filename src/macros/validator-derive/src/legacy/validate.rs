use crate::legacy::internals::{
  input::{attr, check, Data, Field, Input, Style, Variant},
  utils, Context, ExpandResult,
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::DeriveInput;

pub fn derive_validate(input: &DeriveInput) -> ExpandResult {
  let ctx = Context::new();

  let input = match Input::from_derive(&ctx, input) {
    Some(input) => input,
    None => return Err(ctx.check().unwrap_err()),
  };
  check::input(&ctx, &input);
  ctx.check()?;

  let body = generate_body(&input);
  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
  let name = input.ident;

  Ok(quote! {
    impl #impl_generics ::validator::Validate for #name #ty_generics #where_clause {
      fn validate(&self) -> ::std::result::Result<(), ::validator::ValidateError> {
        #body
      }
    }
  })
}

fn generate_body(input: &Input) -> TokenStream {
  match &input.data {
    Data::Enum(variants) => {
      let patterns = variants
        .iter()
        .map(generate_variant_checker)
        .collect::<TokenStream>();

      quote! {
        match &self {
          #patterns
        }
      }
    }
    Data::Struct(style, fields) => {
      let preindex = quote!(self.);
      generate_fields_checker(fields, &preindex, style)
    }
  }
}

fn generate_variant_checker(variant: &Variant) -> TokenStream {
  let variant_ident = &variant.ident;
  let preindex = TokenStream::new();
  let body = generate_fields_checker(&variant.fields, &preindex, &variant.style);

  match variant.style {
    Style::Struct => {
      let members = variant.fields.iter().map(|f| &f.member);
      quote! {
        Self::#variant_ident { #(ref #members),* } => { #body },
      }
    }
    Style::Tuple => {
      let field_names = (0..variant.fields.len())
        .map(|i| syn::Ident::new(&format!("__field{}", i), Span::call_site()));
      quote! {
        Self::#variant_ident(#(ref #field_names),*) => { #body },
      }
    }
    Style::Unit => {
      quote! {
        Self::#variant_ident => Ok(()),
      }
    }
  }
}

fn needs_add_checkers(fields: &[Field]) -> bool {
  for field in fields.iter() {
    if field.attrs.requires_extend() {
      return true;
    }
  }
  false
}

fn generate_fields_checker(fields: &[Field], preindex: &TokenStream, style: &Style) -> TokenStream {
  // Other optimizations
  if fields.is_empty() || !needs_add_checkers(fields) {
    return quote!(Ok(()));
  }

  let mut body = match style {
    Style::Struct => quote! {
      let mut err = ::validator::ValidateError::field_builder();
    },
    Style::Tuple => quote! {
      let mut err = ::validator::ValidateError::slice_builder();
    },
    Style::Unit => unreachable!(),
  };

  for field in fields.iter() {
    body.extend(generate_struct_field_checker(field, preindex, style));
  }
  if !matches!(style, Style::Unit) {
    body.extend(quote!(err.build().into_result()));
  }

  body
}

fn generate_for_length_attrs(attr: &attr::Length) -> (TokenStream, TokenStream, TokenStream) {
  let min = attr
    .min
    .map(|v| quote!( Some(#v) ))
    .unwrap_or_else(|| quote! { None });

  let max = attr
    .max
    .map(|v| quote!( Some(#v) ))
    .unwrap_or_else(|| quote! { None });

  let equal = attr
    .equal
    .map(|v| quote!( Some(#v) ))
    .unwrap_or_else(|| quote! { None });

  (min, max, equal)
}

fn fix_member_name(member: &syn::Member) -> syn::Ident {
  match member {
    syn::Member::Named(data) => data.clone(),
    syn::Member::Unnamed(data) => {
      syn::Ident::new(&format!("__field{}", data.index), Span::call_site())
    }
  }
}

fn generate_struct_field_checker(
  field: &Field,
  preindex: &TokenStream,
  style: &Style,
) -> TokenStream {
  let member = fix_member_name(&field.member);
  let preindex_inner = if field.attrs.allow_optional() {
    quote!()
  } else {
    preindex.clone()
  };

  // we need to build the if chain
  let mut body = TokenStream::new();

  // prioritize builtins before custom implementations
  if field.attrs.allow_nested() && !matches!(style, Style::Unit) {
    let inner_body = match style {
      Style::Struct => {
        let member_as_str = utils::make_lit_str(&member);
        quote!( err.insert(#member_as_str, nested_err); )
      }
      Style::Tuple => quote!( err.insert(nested_err); ),
      Style::Unit => unreachable!(),
    };

    body.extend(quote! {{
      if let Err(nested_err) = <_ as ::validator::Validate>::validate( &#preindex_inner #member ) {
        #inner_body
      }
    }})
  }

  if let Some(length) = field.attrs.length() {
    // TODO: Use the actual parsed LitInt instead.
    let (min, max, equal) = generate_for_length_attrs(length);
    let error_message = utils::make_lit_str("Value must be in between the required length");

    let length_checker =
      quote!( ::validator::extras::validate_length(& #preindex_inner #member, #min, #max, #equal) );

    body.extend(make_base_field_checker(
      &error_message,
      length_checker,
      &member,
      style,
    ));
  }

  if let Some(with) = field.attrs.with() {
    let error_message = match field.attrs.custom_error_msg() {
      Some(data) => data.clone(),
      None => utils::make_lit_str(format_args!(
        "Invalid value for `{}` field",
        field.member_display()
      )),
    };

    body.extend(make_base_field_checker(
      &error_message,
      quote!( #with(&#preindex_inner #member) ),
      &member,
      style,
    ));
  }

  if field.attrs.allow_optional() {
    quote! {
      if let Some(#member) = #preindex #member . as_ref() {
        #body
      }
    }
  } else {
    body
  }
}

fn make_base_field_checker(
  error_message: &syn::LitStr,
  checker_expr: TokenStream,
  member: &syn::Ident,
  style: &Style,
) -> TokenStream {
  let mut inner_body = TokenStream::new();
  if matches!(style, Style::Unit) {
    return inner_body;
  }

  inner_body.extend(quote! {
    let mut msg = ::validator::ValidateError::msg_builder();
    msg.insert(#error_message);
  });

  inner_body.extend(match style {
    Style::Struct => {
      let member_as_str = utils::make_lit_str(member);
      quote!( err.insert(#member_as_str, msg.build()); )
    }
    Style::Tuple => quote!( err.insert(msg.build()); ),
    Style::Unit => unreachable!(),
  });

  quote! {{
    if !#checker_expr {
      #inner_body
    }
  }}
}
