use diesel_migrations::{embed_migrations, EmbeddedMigrations};

mod pool;

#[doc(inline)]
pub use capwat_diesel::prelude;
pub use pool::*;
pub mod entity;
pub mod schema;

pub use capwat_diesel::Pool as RawPool;

pub const MIGRATIONS: EmbeddedMigrations =
    embed_migrations!("../../migrations");
