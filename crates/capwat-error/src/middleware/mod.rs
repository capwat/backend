#![expect(
    deprecated,
    reason = "`Context` is needed because error_stack still uses Context for compatibility reasons"
)]
use capwat_api_types::ErrorCategory;
use error_stack::{Context, Report};
use std::any::Any;
use std::any::TypeId;
use std::panic::Location;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::RwLock;

use crate::Error;

#[path = "std.rs"]
mod std_middleware;

/// Contains internal crate for `capwat-error`, error-stack.
pub mod impls {
    pub use error_stack::{Context, Report};
}

type StoredMiddleware = Arc<
    dyn Fn(Box<dyn Context + 'static>, Location<'static>, &mut ErrorCategory) -> Report
        + Send
        + Sync,
>;

// since rust tests ran in multiple threads, we could use thread_local instead
static IS_INTERNAL_REGISTERED: OnceLock<()> = OnceLock::new();
static MIDDLEWARES: RwLock<Vec<(TypeId, StoredMiddleware)>> = RwLock::new(Vec::new());

impl Error {
    // TODO: Explain what it does making an error object and why users MUST USE
    //       Report::new_without_location and change_context_retain_location.
    pub fn install_middleware<T: Context + Any + Send + Sync + 'static>(
        middleware: impl Fn(T, Location<'static>, &mut ErrorCategory) -> Report + Send + Sync + 'static,
    ) {
        // this is to avoid overflowing the stack.
        if IS_INTERNAL_REGISTERED.get().is_none() {
            install_internal_middlewares();
        }

        let mut lock = MIDDLEWARES
            .write()
            .unwrap_or_else(|_| unreachable!("Hook is posioned."));

        let t = TypeId::of::<T>();
        let exists = lock.iter().any(|(id, _)| t == *id);
        if exists {
            panic!("{} is already installed", std::any::type_name::<T>());
        }

        lock.push((TypeId::of::<T>(), into_boxed_middleware(middleware)));
    }
}

pub(super) fn get_middleware<T: Context + 'static>() -> Option<StoredMiddleware> {
    let lock = MIDDLEWARES
        .read()
        .unwrap_or_else(|_| unreachable!("Hook is posioned."));

    let t = TypeId::of::<T>();
    for (id, middleware) in lock.iter() {
        if *id == t {
            return Some(middleware.clone());
        }
    }

    None
}

pub(super) fn install_internal_middlewares() {
    // do not register again
    if IS_INTERNAL_REGISTERED.get().is_some() {
        return;
    }

    if IS_INTERNAL_REGISTERED.set(()).is_ok() {
        std_middleware::install_middleware();
    }
}

#[track_caller]
fn into_boxed_middleware<T: Context + Any + 'static>(
    middleware: impl Fn(T, Location<'static>, &mut ErrorCategory) -> Report + Send + Sync + 'static,
) -> StoredMiddleware {
    Arc::new(
        #[track_caller]
        move |error: Box<dyn Context>, location: Location<'_>, category: &mut ErrorCategory| {
            // SAFETY: Just checked whether we are pointing to the correct type
            //         from the get_middleware type unless someone maintaining this
            //         code is assumed to be brain dead.
            //
            // Rust won't let us use Box::downcast_unchecked, so we need to copy
            // the entire code of it.
            let error = unsafe {
                let (raw, alloc): (*mut dyn Context, _) = Box::into_raw_with_allocator(error);
                Box::from_raw_in(raw as *mut T, alloc)
            };
            middleware(*error, location, category)
        },
    )
}
