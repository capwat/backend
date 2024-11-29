pub mod users {
    use argon2::password_hash::{rand_core::OsRng, SaltString};
    use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
    use capwat_error::{ApiError, ApiErrorCategory, Error, Result};
    use capwat_model::instance_settings::InstanceSettings;
    use capwat_model::User;
    use thiserror::Error;

    pub fn check_email_status(
        user: &User,
        settings: &InstanceSettings,
    ) -> std::result::Result<(), ApiError> {
        // maybe that user has registered before in that instance but the
        // instance administrator decided to require all users to have their
        // own registered email later on.
        if user.email.is_none() && settings.require_email_registration {
            return Err(ApiError::new(ApiErrorCategory::NoEmailAddress)
                .message("Please input your email address to continue using Capwat"));
        }

        if user.email.is_some() && !user.email_verified && settings.require_email_verification {
            return Err(ApiError::new(ApiErrorCategory::EmailVerificationRequired)
                .message("Please verify your email address to continue using Capwat"));
        }

        Ok(())
    }

    #[derive(Debug, Error)]
    #[error("Failed to verify password hashes")]
    pub struct VerifyPasswordError;

    #[derive(Debug, Error)]
    #[error("Failed to generate password hash")]
    pub struct HashPasswordError;

    pub fn verify_pasword(password: &str, hash: &str) -> Result<bool> {
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::default(),
        );

        let hash = PasswordHash::new(hash).map_err(|e| {
            Error::unknown(VerifyPasswordError)
                .attach_printable("unable to parse password hash")
                .attach_printable(format!("info: {e}"))
        })?;

        match argon2.verify_password(password.as_bytes(), &hash) {
            Ok(..) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(error) => Err(Error::unknown_generic(VerifyPasswordError)
                .attach_printable(format!("info: {error}"))),
        }
    }

    pub fn hash_password(password: &str) -> Result<String, HashPasswordError> {
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::default(),
        );

        let salt = SaltString::generate(&mut OsRng);
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| Error::unknown(HashPasswordError).attach_printable(e.to_string()))?;

        Ok(password_hash.to_string())
    }

    #[must_use]
    pub fn validate_password(password: &str) -> bool {
        const MIN_PASSWORD_LEN: usize = 10;
        // the more, the better!
        const MAX_PASSWORD_LEN: usize = 64;

        let pass_len = password.len();
        (MIN_PASSWORD_LEN..=MAX_PASSWORD_LEN).contains(&pass_len)
    }
}

pub mod time;
pub use self::time::ConsistentRuntime;

#[cfg(test)]
pub mod test {
    use crate::App;
    use axum_test::TestServer;
    use capwat_model::instance_settings::InstanceSettings;
    use capwat_postgres::queries::instance_settings::InstanceSettingsPgImpl;
    use capwat_vfs::{backend::InMemoryFs, Vfs};
    use tracing::info;

    pub async fn build_test_server() -> (TestServer, App) {
        let vfs = Vfs::new(InMemoryFs::new());
        let _ = capwat_utils::env::load_dotenv(&Vfs::new_std());

        capwat_tracing::init_for_tests();
        capwat_postgres::install_error_middleware();

        let app = App::new_for_tests(vfs).await;

        info!("setting up local instance settings");
        let mut conn = app.db_write().await.unwrap();
        InstanceSettings::setup_local(&mut conn).await.unwrap();
        conn.commit().await.unwrap();

        info!("initializing test server");
        let router = crate::controllers::build_axum_router(app.clone());
        let router = crate::middleware::apply(router);
        (TestServer::new(router).unwrap(), app)
    }
}
