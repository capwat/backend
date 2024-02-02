use error_stack::{Context, Report};
use tracing_error::SpanTrace;

pub mod ext;
#[cfg(feature = "main-full")]
mod ext_impl;

pub use capwat_types_common::error::Error as Category;
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct Error {
    category: Category,
    report: Option<Report>,
    trace: SpanTrace,
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self, f)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.category.fmt(f)?;
        if let Some(report) = self.report.as_ref() {
            writeln!(f, ":\n{report:?}")?;
        }
        std::fmt::Display::fmt(&self.trace, f)
    }
}

impl Error {
    #[must_use]
    pub fn new(category: Category) -> Self {
        Self { category, report: None, trace: SpanTrace::capture() }
    }

    #[must_use]
    pub fn internal(context: impl Context) -> Self {
        Self {
            category: Category::Internal,
            report: Some(Report::new(context).as_any()),
            trace: SpanTrace::capture(),
        }
    }

    #[must_use]
    pub fn internal_with_report(report: Report<impl Context>) -> Self {
        Self {
            category: Category::Internal,
            report: Some(report.as_any()),
            trace: SpanTrace::capture(),
        }
    }

    #[must_use]
    pub fn from_context(category: Category, context: impl Context) -> Self {
        Self {
            category,
            report: Some(Report::new(context).as_any()),
            trace: SpanTrace::capture(),
        }
    }

    #[must_use]
    pub fn from_report(
        category: Category,
        report: Report<impl Context>,
    ) -> Self {
        Self {
            category,
            report: Some(report.as_any()),
            trace: SpanTrace::capture(),
        }
    }
}

impl Error {
    #[must_use]
    pub fn as_category(&self) -> &Category {
        &self.category
    }

    #[must_use]
    pub fn change_context(mut self, context: impl Context) -> Self {
        self.report = Some(
            if let Some(report) = self.report {
                report.change_context(context)
            } else {
                Report::new(context)
            }
            .as_any(),
        );
        self
    }

    #[must_use]
    pub fn change_category(self, category: Category) -> Self {
        Error { category, report: self.report, trace: self.trace }
    }

    #[must_use]
    pub fn downcast_ref<F: Context>(&self) -> Option<&F> {
        self.report.as_ref().and_then(|v| v.downcast_ref::<F>())
    }
}
