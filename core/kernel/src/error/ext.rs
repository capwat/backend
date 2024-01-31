use error_stack::{Context, Report};
use thiserror::Error;

use super::{Category, Error};

pub trait ResultExt<T> {
    fn change_context<C: Context>(self, context: C) -> Result<T, Error>;
    fn change_category(self, category: Category) -> Result<T, Error>;
}

impl<T> ResultExt<T> for std::result::Result<T, Error> {
    fn change_context<C: Context>(self, context: C) -> Result<T, Error> {
        self.map_err(|e| e.change_context(context))
    }

    fn change_category(self, category: Category) -> Result<T, Error> {
        self.map_err(|e| e.change_category(category))
    }
}

pub trait ErrorExt<T> {
    fn with_error(self, category: Category) -> Result<T, Error>;
    fn into_error(self) -> Result<T, Error>;
}

// This is for `error-stack` result types
pub trait ErrorExt2<T> {
    fn with_error(self, category: Category) -> Result<T, Error>;
    fn into_error(self) -> Result<T, Error>;
}

impl<T, C: Context> ErrorExt<T> for std::result::Result<T, C> {
    fn with_error(self, category: Category) -> Result<T, Error> {
        self.map_err(|e| Error::from_context(category, e))
    }

    fn into_error(self) -> Result<T, Error> {
        self.map_err(|e| Error::from_context(Category::Internal, e))
    }
}

impl<T, C: Context> ErrorExt2<T> for error_stack::Result<T, C> {
    fn with_error(self, category: Category) -> Result<T, Error> {
        self.map_err(|e| Error::from_report(category, e))
    }

    fn into_error(self) -> Result<T, Error> {
        self.map_err(|e| Error::from_report(Category::Internal, e))
    }
}

// This is with `IntoError` as an error
pub trait ErrorExt3<T> {
    fn into_error(self) -> Result<T, Error>;
}

impl<T, C: IntoError> ErrorExt3<T> for std::result::Result<T, C> {
    fn into_error(self) -> Result<T, Error> {
        self.map_err(|e| e.into_error())
    }
}

// This is for types that do not allow for implement `impl From<Foo> for Error`
pub trait IntoError {
    fn into_error(self) -> Error;
}

// This is for types that wrapped with error_stack's Report type while
// it preserves the report data.
pub trait ReportIntoError: error_stack::Context {
    fn category(&self) -> Category;
}

impl IntoError for Box<dyn std::error::Error + Send + Sync> {
    fn into_error(self) -> Error {
        #[derive(Debug, Error)]
        #[error("{0}")]
        struct InnerError(Box<dyn std::error::Error + Send + Sync>);
        Error::internal(InnerError(self))
    }
}

impl<T: IntoError> From<T> for Error {
    fn from(value: T) -> Self {
        value.into_error()
    }
}

// SAFETY: As long as the user confirms that it is not Report<()>
impl<T: ReportIntoError> From<Report<T>> for Error {
    fn from(value: Report<T>) -> Self {
        let error_type = unsafe { value.current_context().category() };
        Error::from_report(error_type, value)
    }
}
