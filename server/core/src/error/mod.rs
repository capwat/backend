use error_stack::{Context, Report};
use serde::Serialize;
use std::{
    any::{Any, TypeId},
    fmt::{Debug, Display},
};
use tracing_error::SpanTrace;

pub type AppResult<T> = std::result::Result<T, AppError>;

pub struct AppError {
    error_type: Box<dyn AppErrorType>,
    report: Report<Box<dyn Context>>,
    trace: SpanTrace,
}

mod extras;
pub use extras::*;

impl AppError {
    #[inline]
    #[must_use]
    pub fn new(error_type: impl AppErrorType + 'static, error: impl Context) -> Self {
        Self::from_report(error_type, Report::new(error))
    }

    #[inline]
    #[must_use]
    pub fn from_report(
        error_type: impl AppErrorType + 'static,
        report: Report<impl Context>,
    ) -> Self {
        // SAFETY:
        //
        // `report` variable is not using used internally
        // unless it is being explicit from functions like
        // `downcast_ref` where it evaluates if the "inner type"
        // of the error matches from the generic argument.
        //
        // Also, Report<T> where T is a context contains an array
        // of frames and `PhantomData` is being used to obtain
        // a generic parameter for Report<T>.
        let report = unsafe { std::mem::transmute::<_, Report<Box<dyn Context>>>(report) };
        Self::from_report_internal(error_type, report)
    }

    #[inline]
    #[must_use]
    fn from_report_internal(
        error_type: impl AppErrorType + 'static,
        report: Report<Box<dyn Context>>,
    ) -> Self {
        Self {
            error_type: Box::new(error_type),
            report,
            trace: SpanTrace::capture(),
        }
    }
}

impl AppError {
    #[must_use]
    pub fn as_type(&self) -> &dyn AppErrorType {
        self.error_type.as_ref()
    }

    #[must_use]
    pub fn as_context(&self) -> &dyn Context {
        self.report.as_error()
    }

    #[must_use]
    pub fn change_context(mut self, context: impl Context) -> Self {
        // SAFETY: See from Error::from_report
        self.report = unsafe {
            std::mem::transmute::<_, Report<Box<dyn Context>>>(self.report.change_context(context))
        };
        self
    }

    #[must_use]
    pub fn downcast_type<F: AppErrorType>(&self) -> Option<&F> {
        let target = TypeId::of::<F>();
        if self.error_type.type_id() == target {
            // SAFETY: This is already validated above this block
            unsafe {
                let kind = &*self.error_type as *const dyn AppErrorType;
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

impl std::fmt::Debug for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Error")
            .field("type", &self.error_type)
            .field("report", &self.report)
            .field("trace", &self.trace)
            .finish()
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}:", self.error_type)?;
        writeln!(f, "{:?}", self.report)?;
        std::fmt::Display::fmt(&self.trace, f)
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.error_type
            .json_metadata()
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

pub trait AppErrorType: Debug + Display + Send + Sync + 'static {
    // FIXME: This function is not very performant but at least it is
    // not an "object-safe" trait if we allow for multiple serializers
    // at once with a single method.
    //
    // It is possible to implement with unsafe (using vtables) but I'm
    // too lazy to implement something like this. :)

    /// Internal error metadata in JSON form.
    fn json_metadata(&self) -> serde_json::Result<serde_json::Value>;
}

pub mod prelude {
    use super::*;

    pub trait AppErrorExt<T> {
        fn change_context(self, kind: impl AppErrorType + 'static) -> AppResult<T>;
    }

    impl<T, C> AppErrorExt<T> for Result<T, C>
    where
        C: Context,
    {
        fn change_context(self, kind: impl AppErrorType + 'static) -> AppResult<T> {
            self.map_err(|e| AppError::new(kind, e))
        }
    }

    pub trait AppErrorExt2<T> {
        fn change_context(self, kind: impl AppErrorType + 'static) -> AppResult<T>;
    }

    impl<T, C> AppErrorExt2<T> for error_stack::Result<T, C>
    where
        C: Context,
    {
        fn change_context(self, kind: impl AppErrorType + 'static) -> AppResult<T> {
            self.map_err(|e| AppError::from_report(kind, e))
        }
    }

    impl<T> AppErrorExt2<T> for AppResult<T> {
        fn change_context(self, kind: impl AppErrorType + 'static) -> AppResult<T> {
            self.map_err(|mut e| {
                e.error_type = Box::new(kind);
                e
            })
        }
    }

    pub trait AppErrorExt3<T> {
        fn into_internal_err(self) -> AppResult<T>;
    }

    pub trait AppErrorExt4<T> {
        fn into_internal_err(self) -> AppResult<T>;
    }

    impl<T, C: Context> AppErrorExt3<T> for Result<T, C> {
        fn into_internal_err(self) -> AppResult<T> {
            self.map_err(|e| AppError::new(InternalError, e))
        }
    }

    impl<T, C: Context> AppErrorExt4<T> for error_stack::Result<T, C> {
        fn into_internal_err(self) -> AppResult<T> {
            self.map_err(|e| AppError::from_report(InternalError, e))
        }
    }
}
