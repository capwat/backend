use capwat_db::PgPool;
use capwat_vfs::Vfs;
use jsonwebtoken::{DecodingKey, EncodingKey};
use std::sync::Arc;

/// Inner type of [`App`] object.
///
/// [`App`]: super::App
pub struct AppInner {
    pub config: Arc<capwat_config::Server>,
    pub vfs: Vfs,

    pub(super) primary_db: PgPool,
    pub(super) replica_db: Option<PgPool>,

    pub(super) jwt_encode: EncodingKey,
    pub(super) jwt_decode: DecodingKey,
}
