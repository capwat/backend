use pin_project::pin_project;
use std::future::Future;
use std::task::Poll;
use std::time::Duration;
use tokio::time::Sleep;

/// This extension trait allows to easily create [`SubtleTiming`] directly from
/// a [`Future`] value without having to explicitly declare it with [`SubtleTiming::new`].
pub trait SubtleTimingFutureExt: Future {
    fn subtle_timing(self, duration: Duration) -> SubtleTiming<Self>
    where
        Self: Sized;
}

impl<F: Future> SubtleTimingFutureExt for F {
    fn subtle_timing(self, duration: Duration) -> SubtleTiming<Self> {
        SubtleTiming::new(self, duration)
    }
}

/// It allows to perform an operation and waits until the duration timer
/// is completed or not wait if the duration timer exceeds while the
/// operation is ongoing. This allows for the future to have some kind of
/// consistent execution time.
///
/// This is useful for logging in where response timing matters a lot
/// in cybersecurity manner.
#[pin_project]
#[derive(Debug)]
#[must_use]
pub struct SubtleTiming<F: Future> {
    #[pin]
    future: F,
    /// This is to keep our results if it hasn't reached
    /// our subtle timer duration yet.
    result: Option<F::Output>,
    #[pin]
    sleep: Sleep,
}

impl<F: Future> SubtleTiming<F> {
    #[must_use]
    pub fn new(future: F, duration: Duration) -> Self {
        Self {
            future,
            result: None,
            sleep: tokio::time::sleep(duration),
        }
    }
}

impl<F: Future> Future for SubtleTiming<F> {
    type Output = F::Output;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        // do the future first before sleeping in.
        let mut me = self.project();
        if me.result.is_none() {
            match me.future.poll(cx) {
                Poll::Ready(output) => {
                    *me.result = Some(output);
                }
                Poll::Pending => return Poll::Pending,
            };
        }

        match me.sleep.as_mut().poll(cx) {
            Poll::Ready(..) => Poll::Ready(me.result.take().unwrap()),
            Poll::Pending => Poll::Pending,
        }
    }
}
