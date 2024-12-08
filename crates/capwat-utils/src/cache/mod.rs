use moka::future::Cache;
use std::sync::OnceLock;
use std::time::Duration;

pub struct StaticValueCache<T> {
    inner: OnceLock<Cache<(), T>>,
    time_to_live: Duration,
}

impl<T: Clone + Send + Sync + 'static> StaticValueCache<T> {
    #[must_use]
    pub const fn new(time_to_live: Duration) -> Self {
        Self {
            inner: OnceLock::new(),
            time_to_live,
        }
    }

    #[must_use]
    pub async fn get(&self) -> Option<T> {
        let cached = match self.inner.get() {
            Some(inner) => inner,
            None => {
                let inner = Cache::builder().time_to_live(self.time_to_live).build();
                let _ = self.inner.set(inner);
                self.inner.get().unwrap()
            }
        };
        cached.get(&()).await
    }

    #[must_use]
    pub async fn get_or_set(&self, default: T) -> T {
        if let Some(value) = self.get().await {
            value
        } else {
            self.set(default);
            self.get().await.unwrap()
        }
    }

    pub fn set(&self, value: T) {
        let cached = match self.inner.get() {
            Some(inner) => inner,
            None => {
                let inner = Cache::builder().time_to_live(self.time_to_live).build();
                let _ = self.inner.set(inner);
                self.inner.get().unwrap()
            }
        };
        let _ = cached.insert((), value);
    }
}
