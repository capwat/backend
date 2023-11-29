pub use capwat_types::error::ErrorType;
use error_stack::{Context, Report};
use percent_encoding::NON_ALPHANUMERIC;
use thiserror::Error;
use tracing_error::SpanTrace;

mod ext;
mod impls;

pub use ext::*;

pub struct Error {
  error_type: ErrorType,
  report: Option<Report>,
  trace: SpanTrace,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
  #[must_use]
  pub fn new(error_type: ErrorType) -> Self {
    Self { error_type, report: None, trace: SpanTrace::capture() }
  }

  pub fn from_context(error_type: ErrorType, context: impl Context) -> Self {
    Self {
      error_type,
      report: Some(Report::new(context).as_any()),
      trace: SpanTrace::capture(),
    }
  }

  #[must_use]
  pub fn from_report(
    error_type: ErrorType,
    report: Report<impl Context>,
  ) -> Self {
    Self {
      error_type,
      report: Some(report.as_any()),
      trace: SpanTrace::capture(),
    }
  }
}

impl Error {
  #[must_use]
  pub fn as_type(&self) -> &ErrorType {
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

  #[must_use]
  pub fn change_type(self, error_type: ErrorType) -> Self {
    Error { error_type, report: self.report, trace: self.trace }
  }

  #[must_use]
  pub fn downcast_ref<F: Context>(&self) -> Option<&F> {
    self.report.as_ref().and_then(|v| v.downcast_ref::<F>())
  }
}

impl Error {
  #[must_use]
  pub fn from_tonic(status: &tonic::Status) -> Self {
    #[derive(Debug, Error)]
    #[error("x-error-data metadata from gRPC transmission is invalid")]
    struct InvalidErrorData;

    if let Some(data) = status.metadata().get("x-error-data").and_then(|v| {
      percent_encoding::percent_decode(v.as_bytes()).decode_utf8().ok()
    }) {
      if let Ok(error_type) = serde_json::from_str(&data) {
        return Self::new(error_type);
      }
    }

    Self::new(ErrorType::Internal)
  }

  #[must_use]
  pub fn into_tonic_status(&self) -> tonic::Status {
    use tonic::Code;

    struct Printer<'a>(&'a Error);

    impl<'a> std::fmt::Display for Printer<'a> {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.error_type.message(f)
      }
    }

    let code = match self.error_type {
      ErrorType::ReadonlyMode => Code::Unavailable,
      ErrorType::NotAuthenticated => Code::Unauthenticated,
      ErrorType::Internal | ErrorType::Unknown(..) => Code::Internal,
    };

    // Form an error message for the client
    let message = Printer(self).to_string();

    let mut status = tonic::Status::new(code, message);
    let metadata = status.metadata_mut();

    // Keep the rest of the error data in JSON on a header
    //
    // This line below is very critical for serializing
    // internal errors if serialization with other error
    // types fails.
    if matches!(self.error_type, ErrorType::Internal) {
      let content = serde_json::to_string(&self.error_type)
        .expect("failed to serialize internal error struct");

      let content =
        percent_encoding::percent_encode(content.as_bytes(), NON_ALPHANUMERIC)
          .to_string();

      metadata.insert(
        "x-error-data",
        content.parse().expect("failed to parse encoded error data"),
      );
    } else {
      match serde_json::to_string(&self.error_type) {
        Ok(data) => {
          let content =
            percent_encoding::percent_encode(data.as_bytes(), NON_ALPHANUMERIC)
              .to_string();

          match content.parse() {
            Ok(n) => metadata.insert("x-error-data", n),
            Err(e) => {
              return Error::from_context(ErrorType::Internal, e)
                .into_tonic_status()
            },
          };
        },
        Err(err) => {
          return Error::from_context(ErrorType::Internal, err)
            .into_tonic_status()
        },
      }
    }

    status
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
    self.error_type.fmt(f)?;
    writeln!(f, ": {:?}", self.report)?;
    std::fmt::Display::fmt(&self.trace, f)
  }
}
