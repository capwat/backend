#![expect(
    deprecated,
    reason = "`Context` is needed because error_stack still uses Context for compatibility reasons"
)]
use capwat_api_types::ErrorCategory;
use error_stack::Context;

use crate::Error;

pub trait ResultExt {
    type Ok;
    type Context;

    fn attach<A>(self, attachment: A) -> crate::Result<Self::Ok, Self::Context>
    where
        A: Send + Sync + 'static;

    fn attach_lazy<A, F>(self, attachment: F) -> crate::Result<Self::Ok, Self::Context>
    where
        A: Send + Sync + 'static,
        F: FnOnce() -> A;

    fn attach_printable<A>(self, attachment: A) -> crate::Result<Self::Ok, Self::Context>
    where
        A: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static;

    fn attach_printable_lazy<A, F>(self, attachment: F) -> crate::Result<Self::Ok, Self::Context>
    where
        A: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
        F: FnOnce() -> A;

    fn category(self, category: ErrorCategory) -> crate::Result<Self::Ok, Self::Context>;

    fn change_context<P>(self, context: P) -> crate::Result<Self::Ok, P>
    where
        P: Context;

    fn change_context_lazy<P, F>(self, context: F) -> crate::Result<Self::Ok, P>
    where
        P: Context,
        F: FnOnce() -> P;

    fn erase_context(self) -> crate::Result<Self::Ok>;
}

impl<T, C: Context> ResultExt for std::result::Result<T, C> {
    type Ok = T;
    type Context = C;

    #[track_caller]
    fn attach<A>(self, attachment: A) -> crate::Result<T, C>
    where
        A: Send + Sync + 'static,
    {
        match self {
            Ok(okay) => Ok(okay),
            Err(error) => Err(Error::unknown(error).attach(attachment)),
        }
    }

    #[track_caller]
    fn attach_lazy<A, F>(self, attachment: F) -> crate::Result<T, C>
    where
        A: Send + Sync + 'static,
        F: FnOnce() -> A,
    {
        match self {
            Ok(okay) => Ok(okay),
            Err(error) => Err(Error::unknown(error).attach(attachment())),
        }
    }

    #[track_caller]
    fn attach_printable<A>(self, attachment: A) -> crate::Result<T, C>
    where
        A: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
    {
        match self {
            Ok(okay) => Ok(okay),
            Err(error) => Err(Error::unknown(error).attach_printable(attachment)),
        }
    }

    #[track_caller]
    fn attach_printable_lazy<A, F>(self, attachment: F) -> crate::Result<T, C>
    where
        A: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
        F: FnOnce() -> A,
    {
        match self {
            Ok(okay) => Ok(okay),
            Err(error) => Err(Error::unknown(error).attach_printable(attachment())),
        }
    }

    #[track_caller]
    fn category(self, category: ErrorCategory) -> crate::Result<T, C> {
        match self {
            Ok(okay) => Ok(okay),
            Err(error) => Err(Error::new(category, error)),
        }
    }

    #[track_caller]
    fn change_context<P>(self, context: P) -> crate::Result<T, P>
    where
        P: Context,
    {
        match self {
            Ok(okay) => Ok(okay),
            Err(error) => Err(Error::unknown(error).change_context(context)),
        }
    }

    #[track_caller]
    fn change_context_lazy<P, F>(self, context: F) -> crate::Result<T, P>
    where
        P: Context,
        F: FnOnce() -> P,
    {
        match self {
            Ok(okay) => Ok(okay),
            Err(error) => Err(Error::unknown(error).change_context(context())),
        }
    }

    #[track_caller]
    fn erase_context(self) -> crate::Result<T> {
        match self {
            Ok(okay) => Ok(okay),
            Err(error) => Err(Error::unknown_generic(error)),
        }
    }
}

impl<T, C: Context> ResultExt for crate::Result<T, C> {
    type Ok = T;
    type Context = C;

    #[track_caller]
    fn attach<A>(self, attachment: A) -> crate::Result<T, C>
    where
        A: Send + Sync + 'static,
    {
        // Cannot use .map_err because it will affect the caller location
        match self {
            Ok(okay) => Ok(okay),
            Err(error) => Err(error.attach(attachment)),
        }
    }

    #[track_caller]
    fn attach_lazy<A, F>(self, attachment: F) -> crate::Result<T, C>
    where
        A: Send + Sync + 'static,
        F: FnOnce() -> A,
    {
        // Cannot use .map_err because it will affect the caller location
        match self {
            Ok(okay) => Ok(okay),
            Err(error) => Err(error.attach(attachment())),
        }
    }

    #[track_caller]
    fn attach_printable<A>(self, attachment: A) -> crate::Result<T, C>
    where
        A: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
    {
        // Cannot use .map_err because it will affect the caller location
        match self {
            Ok(okay) => Ok(okay),
            Err(error) => Err(error.attach_printable(attachment)),
        }
    }

    #[track_caller]
    fn attach_printable_lazy<A, F>(self, attachment: F) -> crate::Result<T, C>
    where
        A: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
        F: FnOnce() -> A,
    {
        // Cannot use .map_err because it will affect the caller location
        match self {
            Ok(okay) => Ok(okay),
            Err(error) => Err(error.attach_printable(attachment())),
        }
    }

    #[track_caller]
    fn category(self, category: ErrorCategory) -> crate::Result<T, C> {
        // Cannot use .map_err because it will affect the caller location
        match self {
            Ok(okay) => Ok(okay),
            Err(error) => Err(error.category(category)),
        }
    }

    #[track_caller]
    fn change_context<P>(self, context: P) -> crate::Result<T, P>
    where
        P: Context,
    {
        // Cannot use .map_err because it will affect the caller location
        match self {
            Ok(okay) => Ok(okay),
            Err(error) => Err(error.change_context(context)),
        }
    }

    #[track_caller]
    fn change_context_lazy<P, F>(self, context: F) -> crate::Result<T, P>
    where
        P: Context,
        F: FnOnce() -> P,
    {
        // Cannot use .map_err because it will affect the caller location
        match self {
            Ok(okay) => Ok(okay),
            Err(error) => Err(error.change_context(context())),
        }
    }

    #[track_caller]
    fn erase_context(self) -> crate::Result<T> {
        // Cannot use .map_err because it will affect the caller location
        match self {
            Ok(okay) => Ok(okay),
            Err(error) => Err(error.erase_context()),
        }
    }
}

pub trait NoContextResultExt {
    type Ok;

    fn attach<A>(self, attachment: A) -> crate::Result<Self::Ok>
    where
        A: Send + Sync + 'static;

    fn attach_lazy<A, F>(self, attachment: F) -> crate::Result<Self::Ok>
    where
        A: Send + Sync + 'static,
        F: FnOnce() -> A;

    fn attach_printable<A>(self, attachment: A) -> crate::Result<Self::Ok>
    where
        A: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static;

    fn attach_printable_lazy<A, F>(self, attachment: F) -> crate::Result<Self::Ok>
    where
        A: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
        F: FnOnce() -> A;

    fn category(self, category: ErrorCategory) -> crate::Result<Self::Ok>;

    fn change_context<P>(self, context: P) -> crate::Result<Self::Ok, P>
    where
        P: Context;

    fn change_context_lazy<P, F>(self, context: F) -> crate::Result<Self::Ok, P>
    where
        P: Context,
        F: FnOnce() -> P;

    fn change_context_slient<P>(self, context: P) -> crate::Result<Self::Ok>
    where
        P: Context;

    fn change_context_slient_lazy<P, F>(self, context: F) -> crate::Result<Self::Ok>
    where
        P: Context,
        F: FnOnce() -> P;
}

impl<T> NoContextResultExt for crate::Result<T> {
    type Ok = T;

    #[track_caller]
    fn attach<A>(self, attachment: A) -> crate::Result<T>
    where
        A: Send + Sync + 'static,
    {
        // Cannot use .map_err because it will affect the caller location
        match self {
            Ok(okay) => Ok(okay),
            Err(error) => Err(error.attach(attachment)),
        }
    }

    #[track_caller]
    fn attach_lazy<A, F>(self, attachment: F) -> crate::Result<T>
    where
        A: Send + Sync + 'static,
        F: FnOnce() -> A,
    {
        // Cannot use .map_err because it will affect the caller location
        match self {
            Ok(okay) => Ok(okay),
            Err(error) => Err(error.attach(attachment())),
        }
    }

    #[track_caller]
    fn attach_printable<A>(self, attachment: A) -> crate::Result<T>
    where
        A: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
    {
        // Cannot use .map_err because it will affect the caller location
        match self {
            Ok(okay) => Ok(okay),
            Err(error) => Err(error.attach_printable(attachment)),
        }
    }

    #[track_caller]
    fn attach_printable_lazy<A, F>(self, attachment: F) -> crate::Result<T>
    where
        A: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
        F: FnOnce() -> A,
    {
        // Cannot use .map_err because it will affect the caller location
        match self {
            Ok(okay) => Ok(okay),
            Err(error) => Err(error.attach_printable(attachment())),
        }
    }

    #[track_caller]
    fn category(self, category: ErrorCategory) -> crate::Result<T> {
        // Cannot use .map_err because it will affect the caller location
        match self {
            Ok(okay) => Ok(okay),
            Err(error) => Err(error.category(category)),
        }
    }

    #[track_caller]
    fn change_context<P>(self, context: P) -> crate::Result<T, P>
    where
        P: Context,
    {
        // Cannot use .map_err because it will affect the caller location
        match self {
            Ok(okay) => Ok(okay),
            Err(error) => Err(error.change_context(context)),
        }
    }

    #[track_caller]
    fn change_context_lazy<P, F>(self, context: F) -> crate::Result<T, P>
    where
        P: Context,
        F: FnOnce() -> P,
    {
        // Cannot use .map_err because it will affect the caller location
        match self {
            Ok(okay) => Ok(okay),
            Err(error) => Err(error.change_context(context())),
        }
    }

    #[track_caller]
    fn change_context_slient<P>(self, context: P) -> crate::Result<T>
    where
        P: Context,
    {
        // Cannot use .map_err because it will affect the caller location
        match self {
            Ok(okay) => Ok(okay),
            Err(error) => Err(error.change_context_slient(context)),
        }
    }

    #[track_caller]
    fn change_context_slient_lazy<P, F>(self, context: F) -> crate::Result<T>
    where
        P: Context,
        F: FnOnce() -> P,
    {
        match self {
            Ok(okay) => Ok(okay),
            Err(error) => Err(error.change_context_slient(context())),
        }
    }
}
