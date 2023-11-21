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

  // As long as we are careful writing this during pre-expansion
  #[allow(clippy::expect_used)]
  pub fn spanned_error<A: ToTokens, T: Display>(&self, obj: A, msg: T) {
    self
      .errors
      .borrow_mut()
      .as_mut()
      .expect("should not give any errors after check")
      .push(syn::Error::new_spanned(obj, msg));
  }

  // As long as we are careful writing this during pre-expansion
  #[allow(clippy::expect_used)]
  pub fn error(&self, err: syn::Error) {
    self
      .errors
      .borrow_mut()
      .as_mut()
      .expect("should not give any errors after check")
      .push(err);
  }

  /// Consume this object, producing a formatted error string if there are errors.
  pub fn check(self) -> syn::Result<()> {
    // As long as we are careful writing this during pre-expansion
    #[allow(clippy::unwrap_used)]
    let mut errors = self.errors.borrow_mut().take().unwrap().into_iter();

    let Some(mut combined) = errors.next() else {
      return Ok(());
    };

    for rest in errors {
      combined.combine(rest);
    }

    Err(combined)
  }
}

impl Drop for Context {
  fn drop(&mut self) {
    assert!(
      !(std::thread::panicking() && self.errors.borrow().is_none()),
      "forgot to check for errors"
    );
  }
}
