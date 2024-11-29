#![expect(
    deprecated,
    reason = "`Context` is needed because error_stack still uses Context for compatibility reasons"
)]
use capwat_api_types::ErrorCategory;
use error_stack::{Context, Report};
use std::panic::Location;

use crate::middleware;

/// Constructs a raw report. This is useful for making error middlewares.
#[track_caller]
pub fn make_report<C: Context>(
    context: C,
    location: Option<Location<'static>>,
    category: &mut ErrorCategory,
) -> Report {
    self::middleware::install_internal_middlewares();

    if let Some(middleware) = self::middleware::get_middleware::<C>() {
        let location = location.unwrap_or(*Location::caller());
        return (*middleware)(Box::new(context), location, category);
    }

    Report::new_without_location(context)
        .attach_location(location)
        .erase_context()
}
