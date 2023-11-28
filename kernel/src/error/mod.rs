use capwat_types::error::{ErrorCategory, ErrorType, Ignored};
use error_stack::{Context, Report};
use tracing_error::SpanTrace;

mod ext;
pub use ext::*;

pub struct Error<T: ErrorCategory> {
  error_type: ErrorType<T>,
  report: Option<Report>,
  trace: SpanTrace,
}

pub type Result<T, E = Ignored> = std::result::Result<T, Error<E>>;

impl<T: ErrorCategory> Error<T> {
  pub fn new(error_type: ErrorType<T>) -> Self {
    Self { error_type, report: None, trace: SpanTrace::capture() }
  }

  pub fn from_context(error_type: ErrorType<T>, context: impl Context) -> Self {
    Self {
      error_type,
      report: Some(Report::new(context).as_any()),
      trace: SpanTrace::capture(),
    }
  }

  pub fn from_report(
    error_type: ErrorType<T>,
    report: Report<impl Context>,
  ) -> Self {
    Self {
      error_type,
      report: Some(report.as_any()),
      trace: SpanTrace::capture(),
    }
  }
}

impl<T: ErrorCategory> Error<T> {
  #[must_use]
  pub fn as_type(&self) -> &ErrorType<T> {
    &self.error_type
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

  pub fn change_type<C: ErrorCategory>(
    self,
    error_type: ErrorType<C>,
  ) -> Error<C> {
    Error { error_type, report: self.report, trace: self.trace }
  }

  #[must_use]
  pub fn downcast_ref<F: Context>(&self) -> Option<&F> {
    self.report.as_ref().and_then(|v| v.downcast_ref::<F>())
  }
}

impl<T: ErrorCategory> std::fmt::Debug for Error<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Error")
      .field("type", &self.error_type)
      .field("report", &self.report)
      .field("trace", &self.trace)
      .finish()
  }
}

impl<T: ErrorCategory> std::fmt::Display for Error<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.error_type.server_message(f)?;
    writeln!(f, ": {:?}", self.report)?;
    std::fmt::Display::fmt(&self.trace, f)
  }
}
