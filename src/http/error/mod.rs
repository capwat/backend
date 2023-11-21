use crate::types;
use error_stack::{Context, Report};
use tracing_error::SpanTrace;

mod impls;

pub mod ext;
pub use ext::*;

pub type Result<T> = std::result::Result<T, Error>;

pub struct Error {
  error_type: types::Error,
  report: Report<Box<dyn Context>>,
  trace: SpanTrace,
}

impl Error {
  #[must_use]
  pub fn from_context(error_type: types::Error, context: impl Context) -> Self {
    Self {
      error_type,
      report: to_any_report(context),
      trace: SpanTrace::capture(),
    }
  }

  #[must_use]
  pub fn from_report(error_type: types::Error, report: Report<impl Context>) -> Self {
    Self {
      error_type,
      report: cast_to_any_report(report),
      trace: SpanTrace::capture(),
    }
  }
}

impl Error {
  #[must_use]
  pub fn as_type(&self) -> &types::Error {
    &self.error_type
  }

  #[must_use]
  pub fn as_context(&self) -> &dyn Context {
    self.report.as_error()
  }

  #[must_use]
  pub fn change_context(mut self, context: impl Context) -> Self {
    self.report = cast_to_any_report(self.report.change_context(context));
    self
  }

  #[must_use]
  pub fn change_type(mut self, error_type: types::Error) -> Self {
    self.error_type = error_type;
    self
  }

  #[must_use]
  pub fn downcast_ref<F: Context>(&self) -> Option<&F> {
    self.report.downcast_ref::<F>()
  }
}

impl std::fmt::Debug for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Error")
      .field("type", &self.error_type)
      .field("report", &self.report)
      .field("trace", &self.trace)
      .finish()
  }
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}: ", &self.error_type)?;
    writeln!(f, "{:?}", self.report)?;
    std::fmt::Display::fmt(&self.trace, f)
  }
}

fn cast_to_any_report(report: Report<impl Context>) -> Report<Box<dyn Context>> {
  unsafe { std::mem::transmute::<_, Report<Box<dyn Context>>>(report) }
}

fn to_any_report(context: impl Context) -> Report<Box<dyn Context>> {
  unsafe { std::mem::transmute::<_, Report<Box<dyn Context>>>(Report::new(context)) }
}
