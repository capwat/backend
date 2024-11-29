use capwat_api_types::ErrorCategory;
use error_stack::Report;
use tracing::Span;

/// Tag used for [`Error`] to indicate that this is an
/// error with no context type.
///
/// This is also to allow us to implement convienent functions
/// that will make handling no context and contextual errors.
pub struct NoContext;

pub struct ErrorInner {
    pub category: ErrorCategory,
    pub report: Report,
    pub span: Span,
}

impl ErrorInner {
    #[must_use]
    pub fn boxed(category: ErrorCategory, report: Report) -> Box<Self> {
        Box::new(Self {
            category,
            report,
            span: Span::current(),
        })
    }
}
