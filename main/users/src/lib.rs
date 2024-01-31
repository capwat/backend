use async_trait::async_trait;
use capwat_kernel::entity::{
    self,
    id::{AnyMarker, Id, UserMarker},
};
use capwat_kernel::error::ext::ErrorExt3;
use capwat_kernel::Result;
use capwat_kernel::{domain, driver::Snowflake};
use std::sync::{atomic::AtomicU64, Arc};

use database::Database;
use database::{prelude::*, schema};

#[derive(Debug)]
pub struct FakeSnowflake(AtomicU64);

impl FakeSnowflake {
    pub fn new() -> Self {
        Self(AtomicU64::new(1))
    }
}

#[async_trait]
impl Snowflake for FakeSnowflake {
    async fn next_id(&self) -> Result<Id<AnyMarker>> {
        let id = self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(Id::new(id))
    }
}

pub struct ServiceImpl {
    pool: Database,
    snowflake: Arc<dyn Snowflake>,
}

impl ServiceImpl {
    #[must_use]
    pub fn new(pool: Database, snowflake: Arc<dyn Snowflake>) -> Self {
        Self { pool, snowflake }
    }
}

#[async_trait]
impl domain::users::Service for ServiceImpl {
    async fn create(
        &self,
        input: domain::users::CreateUser<'_>,
    ) -> Result<entity::User> {
        use schema::users;

        let mut conn = self.pool.write_defaults().await?;
        let id = self.snowflake.next_id().await?.cast::<UserMarker>();

        let user = diesel::insert_into(users::table)
            .values((
                users::id.eq(id),
                users::name.eq(input.name.into_string()),
                users::email.eq(input.email.into_opt_string()),
                users::password_hash.eq(input.password_hash.into_string()),
            ))
            .get_result::<database::entity::User>(&mut *conn)
            .await
            .into_error()?;

        conn.commit().await?;

        Ok(user.into())
    }

    async fn find_by_id(
        &self,
        id: Id<UserMarker>,
    ) -> Result<Option<entity::User>> {
        use schema::users;

        let mut conn = self.pool.read_prefer_primary().await?;
        let user = users::table
            .filter(users::id.eq(id))
            .get_result::<database::entity::User>(&mut *conn)
            .await
            .optional()
            .into_error()?;

        Ok(user.map(|v| v.into()))
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<entity::User>> {
        use schema::users;

        let mut conn = self.pool.read_prefer_primary().await?;
        let user = users::table
            .filter(users::name.eq(name))
            .get_result::<database::entity::User>(&mut *conn)
            .await
            .optional()
            .into_error()?;

        Ok(user.map(|v| v.into()))
    }
}
