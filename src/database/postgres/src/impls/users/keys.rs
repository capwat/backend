use capwat_error::{ext::ResultExt, Result};
use capwat_model::{
    diesel::schema::user_keys,
    id::UserId,
    user::{InsertUserKeys, UserKeys},
};
use chrono::Utc;
use diesel::{
    query_dsl::methods::{FilterDsl, OrderDsl},
    ExpressionMethods,
};
use diesel_async::RunQueryDsl;
use thiserror::Error;

use crate::pool::PgConnection;

pub trait UserKeysPgImpl {
    /// Gets the current collection of user's keys at the moment.
    ///
    /// **NOTE**:
    /// These keys do not guarantee that they are ready to use or expired.
    async fn get_current(conn: &mut PgConnection<'_>, user_id: UserId) -> Result<UserKeys>;
}

impl UserKeysPgImpl for UserKeys {
    #[tracing::instrument(skip_all, name = "query.user_keys.get_current")]
    async fn get_current(conn: &mut PgConnection<'_>, user_id: UserId) -> Result<UserKeys> {
        user_keys::table
            .filter(user_keys::user_id.eq(user_id))
            .order((user_keys::expires_at.desc(), user_keys::id.desc()))
            .get_result::<Self>(&mut *conn)
            .await
            .erase_context()
    }
}

pub trait InsertUserKeysPgImpl {
    async fn insert(&self, conn: &mut PgConnection<'_>) -> Result<UserKeys, InsertUserKeysError>;
}

#[derive(Debug, Error)]
#[error("Could not insert user keys")]
pub struct InsertUserKeysError;

impl InsertUserKeysPgImpl for InsertUserKeys<'_> {
    #[tracing::instrument(skip_all, name = "query.user_keys.get_current")]
    async fn insert(&self, conn: &mut PgConnection<'_>) -> Result<UserKeys, InsertUserKeysError> {
        let now = Utc::now();
        let expires_at = self
            .rotation_frequency
            .get_expiry_timestamp(now.naive_utc());

        diesel::insert_into(user_keys::table)
            .values((
                user_keys::user_id.eq(self.user_id),
                user_keys::public_key.eq(self.public_key),
                user_keys::encrypted_secret_key.eq(self.encrypted_secret_key),
                user_keys::expires_at.eq(expires_at),
            ))
            .get_result(&mut *conn)
            .await
            .change_context(InsertUserKeysError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impls::test_utils;

    use capwat_crypto::curve25519;
    use chrono::Duration;
    use diesel_async::RunQueryDsl;

    async fn generate_keypair() -> Result<curve25519::KeyPair> {
        tokio::task::spawn_blocking(|| curve25519::KeyPair::generate())
            .await
            .erase_context()
    }

    #[capwat_macros::postgres_query_test]
    async fn should_get_current() -> Result<()> {
        let (pool, _) = test_utils::prepare_env().await?;
        let user = test_utils::prepare_user(&pool, None).await?;

        let mut conn = pool.acquire().await?;
        let old_keys = UserKeys::get_current(&mut conn, user.id).await?;

        // User keys should not be changed but we're going to assume that
        // their key is expired.
        diesel::update(user_keys::table)
            .set(user_keys::expires_at.eq((Utc::now() - Duration::weeks(1)).naive_utc()))
            .execute(&mut *conn)
            .await
            .erase_context()?;

        let keypair = generate_keypair().await?;
        InsertUserKeys::builder()
            .user_id(user.id)
            .rotation_frequency(user.key_rotation_frequency)
            .public_key(&keypair.public_key.serialize().to_string())
            .encrypted_secret_key(&keypair.secret_key.serialize())
            .build()
            .insert(&mut conn)
            .await?;

        let result = UserKeys::get_current(&mut conn, user.id).await?;
        assert_ne!(result.id, old_keys.id);

        Ok(())
    }

    mod insert {
        use super::*;

        #[capwat_macros::postgres_query_test]
        async fn should_insert() -> Result<()> {
            let (pool, _) = test_utils::prepare_env().await?;
            let user = test_utils::prepare_user(&pool, None).await?;
            let keypair = generate_keypair().await?;

            let mut conn = pool.acquire().await?;
            InsertUserKeys::builder()
                .user_id(user.id)
                .rotation_frequency(user.key_rotation_frequency)
                .public_key(&keypair.public_key.serialize().to_string())
                // we're lazy to encrypt with AEAD
                .encrypted_secret_key(&keypair.secret_key.serialize())
                .build()
                .insert(&mut conn)
                .await?;

            Ok(())
        }
    }
}
