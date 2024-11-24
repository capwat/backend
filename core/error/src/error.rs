#![expect(
    deprecated,
    reason = "`Context` is needed because error_stack still uses Context for compatibility reasons"
)]
use capwat_api_types::ErrorCategory;
use core::{fmt, marker::PhantomData};
use error_stack::Context;
use tracing::Span;

use crate::{
    context::make_report,
    internal::{ErrorInner, NoContext},
};

pub struct Error<C = NoContext> {
    pub(crate) inner: Box<ErrorInner>,
    pub(crate) _phantom: PhantomData<C>,
}

// constructors
impl<C> Error<C> {
    #[must_use]
    #[track_caller]
    pub fn new(category: ErrorCategory, context: C) -> Self
    where
        C: Context,
    {
        Self {
            inner: ErrorInner::boxed(category, make_report(context, None)),
            _phantom: PhantomData,
        }
    }

    #[must_use]
    #[track_caller]
    pub fn new_generic(category: ErrorCategory, context: C) -> Error
    where
        C: Context,
    {
        Error {
            inner: ErrorInner::boxed(category, make_report(context, None)),
            _phantom: PhantomData,
        }
    }

    #[must_use]
    #[track_caller]
    pub fn unknown(context: C) -> Self
    where
        C: Context,
    {
        Self::new(ErrorCategory::Unknown, context)
    }

    #[must_use]
    #[track_caller]
    pub fn unknown_generic(context: C) -> Error
    where
        C: Context,
    {
        Self::new_generic(ErrorCategory::Unknown, context)
    }
}

// getters and setters
impl<C> Error<C> {
    #[must_use]
    #[track_caller]
    pub fn attach<A>(mut self, attachment: A) -> Self
    where
        A: Send + Sync + 'static,
    {
        self.inner.report = self.inner.report.attach(attachment);
        self
    }

    #[must_use]
    #[track_caller]
    pub fn attach_printable<A>(mut self, attachment: A) -> Self
    where
        A: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        self.inner.report = self.inner.report.attach_printable(attachment);
        self
    }

    #[must_use]
    pub fn downcast_ref<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.inner.report.downcast_ref::<T>()
    }

    #[must_use]
    pub fn has<T: Send + Sync + 'static>(&self) -> bool {
        self.downcast_ref::<T>().is_some()
    }

    #[must_use]
    pub fn get_category(&self) -> &ErrorCategory {
        &self.inner.category
    }

    #[must_use]
    pub fn category(mut self, category: ErrorCategory) -> Self {
        self.inner.category = category;
        self
    }

    #[must_use]
    #[track_caller]
    pub fn change_context<N>(mut self, context: N) -> Error<N>
    where
        N: Context,
    {
        self.inner.report = self.inner.report.change_context_slient(context);
        Error {
            inner: self.inner,
            _phantom: PhantomData,
        }
    }

    #[must_use]
    #[track_caller]
    pub fn change_context_slient<N>(mut self, context: N) -> Self
    where
        N: Context,
    {
        self.inner.report = self.inner.report.change_context_slient(context);
        Self {
            inner: self.inner,
            _phantom: PhantomData,
        }
    }

    #[must_use]
    pub fn current_context(&self) -> &C
    where
        C: Context,
    {
        self.inner
            .report
            .downcast_ref()
            .unwrap_or_else(|| unreachable!())
    }

    #[must_use]
    pub fn erase_context(self) -> Error {
        Error {
            inner: self.inner,
            _phantom: PhantomData,
        }
    }

    #[must_use]
    pub fn span(&self) -> &Span {
        &self.inner.span
    }
}

impl<C> From<Error<C>> for Error
where
    C: Context,
{
    #[track_caller]
    fn from(value: Error<C>) -> Self {
        value.erase_context()
    }
}

impl<C: Context> From<C> for Error {
    fn from(value: C) -> Self {
        Error::unknown_generic(value)
    }
}

impl<C: Context> From<C> for Error<C> {
    fn from(value: C) -> Self {
        Error::unknown(value)
    }
}
