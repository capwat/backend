use super::Context;

pub mod attr;
pub mod check;

pub struct Input<'a> {
  pub ident: syn::Ident,
  pub data: Data<'a>,
  pub generics: &'a syn::Generics,
  pub original: &'a syn::DeriveInput,
}

pub enum Data<'a> {
  Enum(Vec<Variant<'a>>),
  Struct(Style, Vec<Field<'a>>),
}

impl<'a> Data<'a> {
  fn from_derive_data(ctx: &Context, input: &'a syn::DeriveInput) -> Option<Self> {
    match &input.data {
      syn::Data::Struct(data) => {
        let data = Self::from_fields(ctx, &data.fields);
        Some(Self::Struct(data.0, data.1))
      }
      syn::Data::Enum(data) => Some(Self::from_enum(ctx, data)),
      syn::Data::Union(..) => {
        ctx.spanned_error(input, "validator does not support derive for unions");
        None
      }
    }
  }

  fn from_enum(ctx: &Context, data: &'a syn::DataEnum) -> Self {
    let mut variants = Vec::<Variant<'a>>::new();
    for variant in &data.variants {
      let (style, fields) = Self::from_fields(ctx, &variant.fields);
      variants.push(Variant {
        ident: variant.ident.clone(),
        style,
        fields,
        original: variant,
      });
    }
    Self::Enum(variants)
  }

  fn from_fields(ctx: &Context, fields: &'a syn::Fields) -> (Style, Vec<Field<'a>>) {
    match &fields {
      syn::Fields::Named(fields) => (
        Style::Struct,
        Field::from_ast_multiple(ctx, fields.named.iter()),
      ),
      syn::Fields::Unnamed(fields) => (
        Style::Tuple,
        Field::from_ast_multiple(ctx, fields.unnamed.iter()),
      ),
      syn::Fields::Unit => (Style::Unit, Vec::default()),
    }
  }
}

pub struct Variant<'a> {
  pub ident: syn::Ident,
  pub style: Style,
  pub fields: Vec<Field<'a>>,
  pub original: &'a syn::Variant,
}

pub struct Field<'a> {
  pub member: syn::Member,
  pub attrs: attr::Field,
  pub original: &'a syn::Field,
}

impl<'a> Field<'a> {
  pub fn member_display(&self) -> String {
    match &self.member {
      syn::Member::Named(n) => n.to_string(),
      syn::Member::Unnamed(n) => n.index.to_string(),
    }
  }
}

impl<'a> Field<'a> {
  fn from_ast_multiple(ctx: &Context, fields: impl Iterator<Item = &'a syn::Field>) -> Vec<Self> {
    fields
      .enumerate()
      .map(|(idx, field)| Self::from_ast(ctx, idx, field))
      .collect()
  }

  fn from_ast(ctx: &Context, idx: usize, field: &'a syn::Field) -> Self {
    Field {
      member: match &field.ident {
        Some(ident) => syn::Member::Named(ident.clone()),
        None => syn::Member::Unnamed(idx.into()),
      },
      attrs: attr::Field::from_ast(ctx, field),
      original: field,
    }
  }
}

#[derive(Copy, Clone)]
pub enum Style {
  /// Named fields.
  Struct,
  /// Many unnamed fields.
  Tuple,
  /// No fields inside
  Unit,
}

impl<'a> Input<'a> {
  pub fn from_derive(ctx: &Context, input: &'a syn::DeriveInput) -> Option<Self> {
    let data = Data::from_derive_data(ctx, input)?;

    Some(Self {
      ident: input.ident.clone(),
      data,
      generics: &input.generics,
      original: input,
    })
  }
}
