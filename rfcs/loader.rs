use error_stack::Result;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard};

use super::{LoadError, Loader};

#[derive(Clone)]
pub struct Config<T>(Arc<RwLock<T>>);

impl<T: Loader> Config<T> {
  #[tracing::instrument]
  pub fn init() -> Result<Self, LoadError> {
    T::init().map(|v| Self(Arc::new(RwLock::new(v))))
  }

  #[tracing::instrument(skip(self))]
  pub async fn get(&self) -> RwLockReadGuard<'_, T> {
    self.0.read().await
  }
}

impl<T: Loader> Debug for Config<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if let Ok(data) = self.0.try_read() {
      data.fmt(f)
    } else {
      write!(f, "{}(<locked>)", T::name())
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::hint::black_box;

  #[tokio::test]
  async fn test_display_fmt() {
    #[derive(Debug)]
    struct Sample;

    impl Loader for Sample {
      fn name() -> &'static str {
        "Sample"
      }

      fn init() -> Result<Self, LoadError>
      where
        Self: Sized,
      {
        Ok(Self)
      }
    }

    let cfg = Config::<Sample>::init().unwrap();
    assert_eq!("Sample", format!("{cfg:?}"));

    let n = cfg.0.write().await;
    assert_eq!("Sample(<locked>)", format!("{cfg:?}"));
    let _ = black_box(n);
  }
}
