pub mod instance;
pub mod users;

pub use self::instance::*;
pub use self::users::*;

mod util;

#[cfg(test)]
mod test_utils {
    use capwat_error::{ext::ResultExt, Result};
    use capwat_model::{instance::InstanceSettings, user::InsertUser, KeyRotationFrequency, User};

    use super::{InsertUserPgImpl, InstanceSettingsPgImpl};
    use crate::PgPool;

    pub async fn prepare_env() -> Result<(PgPool, InstanceSettings)> {
        let pool = PgPool::build_for_tests().await;

        let mut conn = pool.acquire().await?;
        InstanceSettings::setup_local(&mut conn).await?;

        let settings = InstanceSettings::get_local(&mut conn).await?;
        drop(conn);

        Ok((pool, settings))
    }

    pub async fn prepare_user(
        pool: &PgPool,
        key_rotation_frequency: Option<KeyRotationFrequency>,
    ) -> Result<User> {
        let params = tokio::task::spawn_blocking(|| {
            capwat_crypto::client::generate_register_user_params("testing")
        })
        .await?;

        InsertUser::builder()
            .name("user")
            .email("test@example.com")
            .access_key_hash(&params.access_key_hash.to_string())
            .encrypted_symmetric_key(&params.encrypted_symmetric_key.to_string())
            .salt(&params.salt.to_string())
            .public_key(&params.public_key.to_string())
            .encrypted_secret_key(&params.encrypted_secret_key.to_string())
            .maybe_key_rotation_frequency(key_rotation_frequency)
            .build()
            .insert(&mut pool.acquire().await?)
            .await
            .erase_context()
    }
}
