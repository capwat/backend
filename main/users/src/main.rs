use capwat_kernel::domain::users::{CreateUser, Service};
use database::Database;
use diesel::migration::MigrationSource;
use std::sync::Arc;
use users::{FakeSnowflake, ServiceImpl};

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let pool = database::RawPool::connect_for_tests(
                database::MIGRATIONS
                    .migrations()
                    .expect("Failed to retrieve migrations"),
            )
            .await;

            let pool = Database::from_pools(pool, None).await.unwrap();

            let snowflake = FakeSnowflake::new();
            let users = ServiceImpl::new(pool, Arc::new(snowflake));
            let user = users
                .create(CreateUser {
                    name: "memothelemo".into(),
                    email: None.into(),
                    password_hash: "hello".into(),
                })
                .await
                .unwrap();

            println!("{user:#?}");
        })
}
