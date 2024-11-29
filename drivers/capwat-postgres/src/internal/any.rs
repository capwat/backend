use async_trait::async_trait;
use bb8::RunError;
use capwat_error::{ext::ResultExt, ApiErrorCategory, Result};
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, PoolError};
use diesel_async::AsyncPgConnection;

use crate::{error::AcquireError, pool::PgConnection};

/// Represents any kind of pool that can be used for testing
/// to production. This is the common interface for communicating
/// between two pools we have:
/// - From bb8
/// - From TestPool in tests module
#[async_trait]
pub trait AnyPool: Send + Sync + 'static {
    async fn acquire(&self) -> Result<PgConnection<'_>, AcquireError>;

    fn idle_connections(&self) -> u32;
    fn connections(&self) -> u32;
    fn is_testing(&self) -> bool;

    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

#[async_trait]
impl AnyPool for bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>> {
    async fn acquire(&self) -> Result<PgConnection<'_>, AcquireError> {
        match self.get_owned().await {
            Ok(conn) => Ok(PgConnection::Pooled(conn)),
            result @ Err(RunError::TimedOut | RunError::User(PoolError::ConnectionError(..))) => {
                match result {
                    Err(error) => Err(error)
                        .change_context(AcquireError::Unhealthy)
                        .category(ApiErrorCategory::Outage),
                    _ => unreachable!(),
                }
            }
            Err(error) => Err(error).change_context(AcquireError::General),
        }
    }

    fn idle_connections(&self) -> u32 {
        self.state().idle_connections
    }

    fn connections(&self) -> u32 {
        self.state().connections
    }

    fn is_testing(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_its_type_name_in_name_fn() {
        struct MockPool;

        #[async_trait]
        impl AnyPool for MockPool {
            async fn acquire(&self) -> Result<PgConnection<'_>, AcquireError> {
                unreachable!()
            }

            fn idle_connections(&self) -> u32 {
                unreachable!()
            }

            fn connections(&self) -> u32 {
                unreachable!()
            }

            fn is_testing(&self) -> bool {
                unreachable!()
            }
        }

        assert_eq!(std::any::type_name::<MockPool>(), MockPool.name());
    }
}
