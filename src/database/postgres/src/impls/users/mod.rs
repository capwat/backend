use capwat_error::{
    ext::{NoContextResultExt, ResultExt},
    Result,
};
use capwat_model::{
    diesel::schema::{user_keys, users},
    instance::InstanceSettings,
    user::InsertUser,
    User,
};
use chrono::Utc;
use diesel::{
    dsl::exists, query_dsl::methods::FilterDsl, BoolExpressionMethods, ExpressionMethods,
    OptionalExtension,
};
use diesel_async::RunQueryDsl;
use thiserror::Error;

use super::{
    util::{coalesce, lower},
    InstanceSettingsPgImpl,
};
use crate::pool::PgConnection;

pub mod keys;
pub use self::keys::{InsertUserKeysPgImpl, UserKeysPgImpl};

pub trait UserPgImpl {
    async fn find(conn: &mut PgConnection<'_>, id: i64) -> Result<Option<User>>;
    async fn find_by_login(conn: &mut PgConnection<'_>, entry: &str) -> Result<Option<User>>;

    async fn check_email_taken(conn: &mut PgConnection<'_>, email: &str) -> Result<bool>;
    async fn check_username_taken(conn: &mut PgConnection<'_>, name: &str) -> Result<bool>;
}

impl UserPgImpl for User {
    #[tracing::instrument(skip_all, name = "query.users.find")]
    async fn find(conn: &mut PgConnection<'_>, id: i64) -> Result<Option<User>> {
        users::table
            .filter(users::id.eq(id))
            .get_result(&mut *conn)
            .await
            .optional()
            .erase_context()
    }

    #[tracing::instrument(skip_all, name = "query.users.find")]
    async fn find_by_login(conn: &mut PgConnection<'_>, entry: &str) -> Result<Option<User>> {
        // remember kids, be specific when it comes to NAMES!
        let filter = lower(users::name)
            .eq(entry)
            .or(lower(coalesce(users::email, "_@_@_")).eq(entry.to_lowercase()));

        users::table
            .filter(filter)
            .get_result::<Self>(&mut *conn)
            .await
            .optional()
            .erase_context()
    }

    #[tracing::instrument(skip_all, name = "query.users.is_email_taken")]
    async fn check_email_taken(conn: &mut PgConnection<'_>, email: &str) -> Result<bool> {
        diesel::select(exists(users::table.filter(users::email.eq(email))))
            .get_result::<bool>(&mut *conn)
            .await
            .erase_context()
    }

    #[tracing::instrument(skip_all, name = "query.users.is_username_taken")]
    async fn check_username_taken(conn: &mut PgConnection<'_>, name: &str) -> Result<bool> {
        diesel::select(exists(
            users::table.filter(lower(users::name).eq(name.to_lowercase())),
        ))
        .get_result::<bool>(&mut *conn)
        .await
        .erase_context()
    }
}

#[derive(Debug, Error)]
#[error("Could not insert user")]
pub struct InsertUserError;

pub trait InsertUserPgImpl {
    async fn insert(&self, conn: &mut PgConnection<'_>) -> Result<User, InsertUserError>;
}

impl InsertUserPgImpl for InsertUser<'_> {
    #[tracing::instrument(skip_all, name = "query.users.insert")]
    async fn insert(&self, conn: &mut PgConnection<'_>) -> Result<User, InsertUserError> {
        // set to `None` if the display name specified is empty
        let display_name = if self.display_name.map(|v| !v.is_empty()).unwrap_or_default() {
            self.display_name
        } else {
            None
        };

        let key_rotation_frequency = if let Some(value) = self.key_rotation_frequency {
            value
        } else {
            let settings = InstanceSettings::get_local(conn)
                .await
                .change_context(InsertUserError)
                .attach_printable(
                    "could not get instance settings to get default key rotation frequency",
                )?;

            settings.default_key_rotation_frequency
        };
        let expires_at = key_rotation_frequency.get_expiry_timestamp(Utc::now().naive_utc());

        let user = diesel::insert_into(users::table)
            .values((
                users::name.eq(self.name),
                users::display_name.eq(display_name),
                users::email.eq(self.email),
                users::access_key_hash.eq(self.access_key_hash),
                users::encrypted_symmetric_key.eq(self.encrypted_symmetric_key),
                users::key_rotation_frequency.eq(key_rotation_frequency),
                users::salt.eq(self.salt),
            ))
            .get_result::<User>(&mut *conn)
            .await
            .change_context(InsertUserError)?;

        diesel::insert_into(user_keys::table)
            .values((
                user_keys::user_id.eq(user.id),
                user_keys::public_key.eq(self.public_key),
                user_keys::encrypted_secret_key.eq(self.encrypted_secret_key),
                user_keys::expires_at.eq(expires_at),
            ))
            .execute(&mut *conn)
            .await
            .change_context(InsertUserError)
            .attach_printable("could not insert user's keys")?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impls::test_utils;

    mod find_by_login {
        use super::*;
        use capwat_crypto::client::generate_register_user_params;
        use tokio::task::spawn_blocking;

        #[capwat_macros::postgres_query_test]
        async fn should_found_user_from_email() -> Result<()> {
            let (pool, _) = test_utils::prepare_env().await?;
            let params = spawn_blocking(|| generate_register_user_params("testing")).await?;

            // it should scan usernames in non-case sensitive manner
            let expected = InsertUser::builder()
                .name("user")
                .email("test@example.com")
                .access_key_hash(&params.access_key_hash.to_string())
                .encrypted_symmetric_key(&params.encrypted_symmetric_key.to_string())
                .salt(&params.salt.to_string())
                .public_key(&params.public_key.to_string())
                .encrypted_secret_key(&params.encrypted_secret_key.to_string())
                .build()
                .insert(&mut pool.acquire().await?)
                .await?;

            let result =
                User::find_by_login(&mut pool.acquire().await?, "test@example.com").await?;

            assert!(result.is_some());

            let result = result.unwrap();
            assert_eq!(expected.id, result.id);

            Ok(())
        }

        #[capwat_macros::postgres_query_test]
        async fn should_found_user_from_username() -> Result<()> {
            let (pool, _) = test_utils::prepare_env().await?;
            let params = spawn_blocking(|| generate_register_user_params("testing")).await?;

            // it should scan usernames in non-case sensitive manner
            let expected = InsertUser::builder()
                .name("Macrowave")
                .email("test@example.com")
                .access_key_hash(&params.access_key_hash.to_string())
                .encrypted_symmetric_key(&params.encrypted_symmetric_key.to_string())
                .salt(&params.salt.to_string())
                .public_key(&params.public_key.to_string())
                .encrypted_secret_key(&params.encrypted_secret_key.to_string())
                .build()
                .insert(&mut pool.acquire().await?)
                .await?;

            let result = User::find_by_login(&mut pool.acquire().await?, "macrowave").await?;
            assert!(result.is_some());

            let result = result.unwrap();
            assert_eq!(expected.id, result.id);

            Ok(())
        }
    }

    mod insert {
        use super::*;
        use capwat_model::{user::UserKeys, KeyRotationFrequency};

        #[capwat_macros::postgres_query_test]
        async fn should_insert() -> Result<()> {
            let (pool, _) = test_utils::prepare_env().await?;
            let result = test_utils::prepare_user(&pool, None).await;
            assert!(result.is_ok());
            UserKeys::get_current(&mut pool.acquire().await?, result.unwrap().id).await?;

            Ok(())
        }

        #[capwat_macros::postgres_query_test]
        async fn should_override_key_rotation_freq() -> Result<()> {
            let (pool, _) = test_utils::prepare_env().await?;
            let result =
                test_utils::prepare_user(&pool, Some(KeyRotationFrequency::Weekly)).await?;

            assert_eq!(result.key_rotation_frequency, KeyRotationFrequency::Weekly);
            Ok(())
        }
    }
}
