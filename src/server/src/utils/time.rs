use pin_project::pin_project;
use std::future::Future;
use std::task::Poll;
use std::time::Duration;
use tokio::time::Sleep;

/// This timer allows to perform an operation and waits until the duration
/// timer is completed or not wait if the duration timer exceeds while the
/// operation is ongoing. This allows for the future to have some kind of
/// consistent execution time.
///
/// This is useful for logging in where response timing matters a lot
/// in cybersecurity manner.
#[pin_project]
#[derive(Debug)]
pub struct ConsistentRuntime<F: Future> {
    #[pin]
    future: F,
    result: Option<F::Output>,
    #[pin]
    sleep: Sleep,
}

impl<F: Future> ConsistentRuntime<F> {
    #[must_use]
    pub fn new(duration: Duration, future: F) -> Self {
        Self {
            future,
            result: None,
            sleep: tokio::time::sleep(duration),
        }
    }
}

impl<F: Future> Future for ConsistentRuntime<F> {
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
