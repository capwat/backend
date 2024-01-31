use pin_project_lite::pin_project;
use std::{
    future::Future,
    task::{ready, Poll},
    time::{Duration, Instant},
};

pin_project! {
    #[derive(Debug)]
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct BenchmarkFuture<Fut> {
        #[pin]
        future: Fut,
        instant: Instant,
    }
}

impl<Fut: Future> Future for BenchmarkFuture<Fut> {
    type Output = (Duration, Fut::Output);

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        let this = self.as_mut().project();
        let value = ready!(this.future.poll(cx));

        Poll::Ready((this.instant.elapsed(), value))
    }
}

pub trait CapwatFutureExt: Future {
    fn benchmark(self) -> BenchmarkFuture<Self>
    where
        Self: Sized;
}

impl<F: Future> CapwatFutureExt for F {
    fn benchmark(self) -> BenchmarkFuture<Self>
    where
        Self: Sized,
    {
        BenchmarkFuture { future: self, instant: Instant::now() }
    }
}

pub trait IntoOptionalFuture<T: Future> {
    fn optional(self) -> OptionalFuture<T>;
}

impl<T: Future> IntoOptionalFuture<T> for Option<T> {
    fn optional(self) -> OptionalFuture<T> {
        OptionalFuture::Idle { future: self }
    }
}

pin_project! {
    #[project = OptFutProj]
    #[project_replace = OptFutProjReplace]
    #[derive(Debug)]
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub enum OptionalFuture<Fut> {
        Idle {
            future: Option<Fut>
        },
        Progress {
            #[pin]
            future: Fut,
        },
        Complete,
    }
}

impl<Fut> Future for OptionalFuture<Fut>
where
    Fut: Future,
{
    type Output = Option<Fut::Output>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.as_mut().project() {
            OptFutProj::Idle { future } => {
                let Some(future) = future.take() else {
                    return Poll::Ready(None);
                };

                self.project_replace(OptionalFuture::Progress { future });
                Poll::Pending
            },
            OptFutProj::Progress { future } => {
                let output = ready!(future.poll(cx));
                Poll::Ready(Some(output))
            },
            OptFutProj::Complete => unreachable!(),
        }
    }
}
