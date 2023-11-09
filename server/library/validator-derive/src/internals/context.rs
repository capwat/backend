use quote::ToTokens;
use std::{cell::RefCell, fmt::Display};

#[derive(Default)]
pub struct Context {
    errors: RefCell<Option<Vec<syn::Error>>>,
}

impl Context {
    pub const fn new() -> Self {
        Self {
            errors: RefCell::new(Some(Vec::new())),
        }
    }

    pub fn spanned_error<A: ToTokens, T: Display>(&self, obj: A, msg: T) {
        self.errors
            .borrow_mut()
            .as_mut()
            .expect("should not give any errors after check")
            .push(syn::Error::new_spanned(obj, msg));
    }

    pub fn error(&self, err: syn::Error) {
        self.errors
            .borrow_mut()
            .as_mut()
            .expect("should not give any errors after check")
            .push(err);
    }

    /// Consume this object, producing a formatted error string if there are errors.
    pub fn check(self) -> syn::Result<()> {
        let mut errors = self.errors.borrow_mut().take().unwrap().into_iter();

        let mut combined = match errors.next() {
            Some(first) => first,
            None => return Ok(()),
        };

        for rest in errors {
            combined.combine(rest);
        }

        Err(combined)
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        if !std::thread::panicking() && self.errors.borrow().is_some() {
            panic!("forgot to check for errors");
        }
    }
}
