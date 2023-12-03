pub mod ext;
mod impls;

use error_stack::{Context, Report};
use tracing_error::SpanTrace;

pub use capwat_types::Error as ErrorCategory;
pub type Result<T> = std::result::Result<T, Error>;

pub struct Error {
  category: ErrorCategory,
  report: Option<Report>,
  trace: SpanTrace,
}

impl Error {
  #[must_use]
  pub fn new(category: ErrorCategory) -> Self {
    Self { category, report: None, trace: SpanTrace::capture() }
  }

  #[must_use]
  pub fn from_context(category: ErrorCategory, context: impl Context) -> Self {
    Self {
      category,
      report: Some(Report::new(context).as_any()),
      trace: SpanTrace::capture(),
    }
  }

  #[must_use]
  pub fn from_report(
    category: ErrorCategory,
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
  pub fn as_category(&self) -> &ErrorCategory {
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
  pub fn change_type(self, category: ErrorCategory) -> Self {
    Error { category, report: self.report, trace: self.trace }
  }

  #[must_use]
  pub fn downcast_ref<F: Context>(&self) -> Option<&F> {
    self.report.as_ref().and_then(|v| v.downcast_ref::<F>())
  }
}

impl std::fmt::Debug for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Error")
      .field("type", &self.category)
      .field("report", &self.report)
      .field("trace", &self.trace)
      .finish()
  }
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.category.fmt(f)?;
    writeln!(f, ": {:?}", self.report)?;
    std::fmt::Display::fmt(&self.trace, f)
  }
}

// This is for types that do not allow for implement `impl From<Foo> for Error`
pub trait IntoError {
  fn into_capwat_error(self) -> Error;
}

// This is for types that wrapped with error_stack's Report type while
// it preserves the report data.
pub trait ReportIntoError: error_stack::Context {
  fn category(&self) -> ErrorCategory;
}
