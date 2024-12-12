use capwat_error::{Error, Result};
use std::future::Future;
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, warn};

pub struct Retry<C> {
    builder: RetryBuilder<C>,
    tries: usize,
}

#[derive(Debug, Error)]
#[error("operation failed many times")]
pub struct RetryFailed;

#[allow(deprecated)]
impl<O, E, F: Future<Output = Result<O, E>>, C: FnMut() -> F> Retry<C> {
    #[must_use]
    #[inline(always)]
    pub fn builder(name: &'static str, callback: C) -> RetryBuilder<C> {
        RetryBuilder::new(name, callback)
    }

    pub async fn run(mut self) -> Result<O, RetryFailed> {
        loop {
            self.tries += 1;
            debug!(tries = %self.tries, "(re)trying task {:?}...", self.builder.name);

            let output = (self.builder.callback)().await;
            match output {
                Ok(output) => return Ok(output),
                Err(error) => {
                    warn!(
                        tries = %self.tries,
                        %error,
                        "operation {:?} failed. retrying for {:?}...",
                        self.builder.name, self.builder.wait
                    );
                }
            }

            if self.tries >= self.builder.max_retries {
                return Err(Error::unknown(RetryFailed));
            }
            tokio::time::sleep(self.builder.wait).await;
        }
    }
}

#[must_use]
pub struct RetryBuilder<C> {
    callback: C,
    max_retries: usize,
    name: &'static str,
    wait: Duration,
}

impl<F, C: FnMut() -> F> RetryBuilder<C> {
    const DEFAULT_MAX_RETRIES: usize = 3;
    const DEFAULT_WAIT: Duration = Duration::from_secs(1);

    #[must_use]
    pub fn new(name: &'static str, callback: C) -> Self {
        Self {
            callback,
            max_retries: Self::DEFAULT_MAX_RETRIES,
            name,
            wait: Self::DEFAULT_WAIT,
        }
    }

    #[must_use]
    pub fn max_retries(self, max_retries: usize) -> Self {
        Self {
            max_retries,
            ..self
        }
    }

    #[must_use]
    pub fn wait(self, duration: Duration) -> Self {
        Self {
            wait: duration,
            ..self
        }
    }

    #[must_use]
    pub fn build(self) -> Retry<C> {
        Retry {
            builder: self,
            tries: 0,
        }
    }
}
