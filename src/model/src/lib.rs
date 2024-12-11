mod postgres;

pub mod id;
pub mod instance;
pub mod user;

pub use self::instance::InstanceSettings;
pub use self::user::User;

use diesel_async_migrations::EmbeddedMigrations;

pub const DB_MIGRATIONS: EmbeddedMigrations = diesel_async_migrations::embed_migrations!();
