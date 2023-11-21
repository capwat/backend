use super::Input;
use crate::legacy::internals::{
  input::{attr, Field},
  Context,
};

pub fn input(ctx: &Context, input: &Input<'_>) {
  match &input.data {
    super::Data::Enum(variants) => {
      for variant in variants {
        check_fields(ctx, variant.fields.iter());
      }
    }
    super::Data::Struct(.., fields) => {
      check_fields(ctx, fields.iter());
    }
  }
}

fn check_fields(ctx: &Context, fields: std::slice::Iter<'_, Field<'_>>) {
  fn check_range_like_params<T: Ord + std::fmt::Display>(
    ctx: &Context,
    original: &syn::Field,
    min: Option<T>,
    max: Option<T>,
  ) {
    let min_and_max_cmp = min.as_ref().zip(max.as_ref()).map(|(min, max)| {
      // Making sure that min is not greater/equal than max
      min.cmp(max)
    });

    match min_and_max_cmp {
      Some(std::cmp::Ordering::Equal) => {
        ctx.spanned_error(
          original,
          format_args!(
            "the minimum value is equal to the maximum value, do you mean to use `equal = ...`?",
          ),
        );
      }
      Some(std::cmp::Ordering::Greater) => {
        // It is already checked from is_min_greater
        ctx.spanned_error(
          original,
          format_args!(
            "the minimum value ({}) is greater than its maximum value ({})",
            min.unwrap(),
            max.unwrap()
          ),
        );
      }
      _ => {}
    }
  }

  fn check_conflicting_attrs(ctx: &Context, attrs: &attr::Field, original: &syn::Field) {
    let has_length_attr = attrs.length().is_some();
    let has_ranges_attr = attrs.ranges().is_some();

    // Nested fields must have no builtin stuff on top
    if attrs.allow_nested() && (has_length_attr || has_ranges_attr) {
      return ctx.spanned_error(
        original,
        r#"#[validator(nested)] must have no built-in checks excluding #[validator(with = "...")]"#,
      );
    }

    // You cannot have length and range requirement at the same time
    if has_length_attr && has_ranges_attr {
      return ctx.spanned_error(
        original,
        "#[validator(length(...))] and #[validator(ranges(...))] cannot be used at the same time",
      );
    }

    if let Some(params) = attrs.ranges() {
      check_range_like_params(ctx, original, params.min, params.max);
    }

    if let Some(params) = attrs.length() {
      if params.equal.is_some() && (params.min.is_some() || params.max.is_some()) {
        return ctx.spanned_error(original, format_args!(
          "length attribute has conflicting requirements. `{} = ...` and/or `{} = ...`, or `{} = ...` must be either set",
          attr::PATH_LENGTH_MIN, attr::PATH_LENGTH_MAX, attr::PATH_LENGTH_EQUAL
        ));
      }

      if params.equal.is_none() {
        check_range_like_params(ctx, original, params.min, params.max);
      }
    }
  }

  for field in fields {
    check_conflicting_attrs(ctx, &field.attrs, field.original);
  }
}
