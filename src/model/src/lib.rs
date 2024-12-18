/// PostgreSQL with `sqlx` implementation of `capwat-model`
mod postgres;

pub mod id;
pub mod instance;
pub mod post;
pub mod user;

pub use self::instance::InstanceSettings;
pub use self::user::User;

use capwat_api_types::user::UserFlags;
use sqlx::migrate::Migrator;

pub const DB_MIGRATIONS: Migrator = sqlx::migrate!("./migrations");

#[must_use]
pub fn setup_user_flags(user: &User, _their_aggregates: &self::user::UserAggregates) -> UserFlags {
    let mut flags = UserFlags::empty();
    if user.admin {
        flags |= UserFlags::ADMINISTRATOR;
    }
    flags
}
