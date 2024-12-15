mod postgres;

pub mod id;
pub mod instance;
pub mod post;
pub mod user;

pub use self::instance::InstanceSettings;
pub use self::user::User;

use sqlx::migrate::Migrator;

pub const DB_MIGRATIONS: Migrator = sqlx::migrate!("./migrations");
