use error_stack::{Context, Report};
use std::any::{Any, TypeId};
use tracing_error::SpanTrace;

mod impls;
mod traits;

pub mod ext;
pub use traits::*;

pub type Result<T> = std::result::Result<T, Error>;

pub struct Error {
  error_type: Box<dyn ErrorType>,
  report: Report<Box<dyn Context>>,
  trace: SpanTrace,
}

impl Error {
  #[must_use]
  pub fn from_context(error_type: impl ErrorType, context: impl Context) -> Self {
    Self {
      error_type: Box::new(error_type),
      report: to_any_report(context),
      trace: SpanTrace::capture(),
    }
  }

  #[must_use]
  pub fn from_report(error_type: impl ErrorType, report: Report<impl Context>) -> Self {
    Self {
      error_type: Box::new(error_type),
      report: cast_to_any_report(report),
      trace: SpanTrace::capture(),
    }
  }
}

impl Error {
  #[must_use]
  pub fn as_type(&self) -> &dyn ErrorType {
    self.error_type.as_ref()
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
  pub fn change_type(mut self, error_type: impl ErrorType) -> Self {
    self.error_type = Box::new(error_type);
    self
  }

  #[must_use]
  pub fn downcast_type<F: ErrorType>(&self) -> Option<&F> {
    let target = TypeId::of::<F>();
    if self.error_type.type_id() == target {
      // SAFETY: This is already validated above this block
      unsafe {
        let kind = &*self.error_type as *const dyn ErrorType;
        Some(&*kind.cast::<F>())
      }
    } else {
      None
    }
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
