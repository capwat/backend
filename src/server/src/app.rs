use axum::extract::{FromRequestParts, State};
use capwat_crypto::rsa::{
    self, DecodeRsaPrivateKey, EncodeRsaPrivateKey, EncodeRsaPublicKey, RsaPrivateKey,
};
use capwat_error::ext::{NoContextResultExt, ResultExt};
use capwat_error::Result;
use capwat_postgres::error::{AcquireError, BeginTransactError};
use capwat_postgres::pool::PgConnection;
use capwat_postgres::transaction::Transaction;
use capwat_postgres::PgPool;
use capwat_vfs::{OpenOptions, Vfs};
use jsonwebtoken::{DecodingKey, EncodingKey};
use thiserror::Error;

use std::fmt::Debug;
use std::io::{Read, Write};
use std::ops::Deref;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, warn};

#[derive(Clone, FromRequestParts)]
#[from_request(via(State))]
#[must_use]
pub struct App(Arc<AppInner>);

#[derive(Debug, Error)]
#[error("Could not initialize server application")]
pub struct AppError;

impl App {
    /// Creates a new [`App`] from a given [configuration](capwat_config::Server).
    pub fn new(config: capwat_config::Server, vfs: Vfs) -> Result<Self, AppError> {
        let primary_db = PgPool::build(&config.database, &config.database.primary);
        let replica_db = config
            .database
            .replica
            .as_ref()
            .map(|replica| PgPool::build(&config.database, replica));

        let (jwt_encode_key, jwt_decode_key) = Self::setup_jwt_auth(&vfs, &config)
            .change_context(AppError)
            .attach_printable("could not setup JWT authentication")?;

        let inner = Arc::new(AppInner {
            config: Arc::new(config),
            primary_db,
            replica_db,
            vfs,

            jwt_encode_key,
            jwt_decode_key,
        });

        Ok(Self(inner))
    }

    /// Creates a new [`App`] for testing purposes.
    #[cfg(test)]
    pub async fn new_for_tests(vfs: Vfs) -> Self {
        let primary_db = PgPool::build_for_tests().await;

        let config = capwat_config::Server::for_tests();
        let (jwt_encode_key, jwt_decode_key) = Self::setup_jwt_auth(&vfs, &config)
            .change_context(AppError)
            .attach_printable("could not setup JWT authentication")
            .unwrap();

        Self(Arc::new(AppInner {
            config: Arc::new(config),
            primary_db,
            replica_db: None,
            vfs,

            jwt_encode_key,
            jwt_decode_key,
        }))
    }

    fn setup_jwt_auth(
        vfs: &Vfs,
        config: &capwat_config::Server,
    ) -> Result<(EncodingKey, DecodingKey)> {
        fn read_jwt_priv_key_file(
            vfs: &Vfs,
            config: &capwat_config::server::Jwt,
        ) -> Result<RsaPrivateKey> {
            let mut buffer = String::with_capacity(rsa::BITS);
            let now = Instant::now();

            debug!("Reading JWT private key file...");
            let mut file = OpenOptions::new()
                .create(true)
                .truncate(false)
                .read(true)
                .write(true)
                .open(vfs, &config.private_key_file)?;

            let bytes_read = file.read_to_string(&mut buffer)?;
            let rsa_key = if bytes_read > 0 {
                RsaPrivateKey::from_pkcs1_pem(&buffer)?
            } else {
                warn!("JWT private key file is missing. Generating new JWT private key (this will invalidate all user sessions)...");
                let new_priv_key = rsa::generate_keypair()?.1;
                file.write_all(new_priv_key.to_pkcs1_pem(rsa::LineEnding::LF)?.as_bytes())?;

                info!(
                    "Created JWT private key file: {}",
                    config.private_key_file.display()
                );
                new_priv_key
            };

            let elapsed = now.elapsed();
            debug!(?elapsed, "Reading JWT private key file done");

            Ok(rsa_key)
        }

        let jwt = &config.auth.jwt;
        let priv_key = read_jwt_priv_key_file(vfs, jwt)
            .attach_printable("could not generate JWT key files")?;

        let priv_key_buffer = priv_key.to_pkcs1_pem(rsa::LineEnding::LF)?;
        let pub_key_buffer = priv_key.to_public_key().to_pkcs1_pem(rsa::LineEnding::LF)?;

        let enc = EncodingKey::from_rsa_pem(priv_key_buffer.as_bytes())
            .attach_printable("could not read RSA PEM of private key file")?;

        let dec = DecodingKey::from_rsa_pem(pub_key_buffer.as_bytes())
            .attach_printable("unexpected to unable to read RSA PEM from public key")?;

        Ok((enc, dec))
    }
}

impl App {
    /// Obtains a read/write database connection from the primary database pool.
    #[tracing::instrument(skip_all, name = "app.db_write")]
    pub async fn db_write(&self) -> Result<Transaction<'_>, BeginTransactError> {
        self.primary_db.begin_default().await
    }

    /// Obtains a readonly database connection from the replica
    /// pool or primary pool whichever is possible to obtain.
    ///
    /// The replica pool will be the first to obtain, if not,
    /// then the primary pool will be obtained instead.
    #[tracing::instrument(skip_all, name = "app.db_read")]
    pub async fn db_read(&self) -> Result<PgConnection<'_>, AcquireError> {
        let Some(replica_pool) = self.replica_db.as_ref() else {
            return self.primary_db.acquire().await;
        };

        match replica_pool.acquire().await {
            Ok(connection) => Ok(connection),
            Err(error) => {
                warn!(%error, "Replica database is not available, falling back to primary");
                self.primary_db.acquire().await
            }
        }
    }

    /// Obtains a readonly database connection from the primary pool.
    ///
    /// If the primary pool is not available, the replica pool will
    /// be used instead to obtain the connection.
    #[tracing::instrument(skip_all, name = "app.db_read_prefer_primary")]
    pub async fn db_read_prefer_primary(&self) -> Result<PgConnection<'_>, AcquireError> {
        let Some(replica_pool) = self.replica_db.as_ref() else {
            return self.primary_db.acquire().await;
        };

        match self.primary_db.acquire().await {
            Ok(connection) => Ok(connection),
            Err(error) => {
                warn!(%error, "Primary database is not available, falling back to replica");
                replica_pool.acquire().await
            }
        }
    }
}

impl Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("config", &self.config)
            .field("primary_db", &self.primary_db)
            .field("replica_db", &self.replica_db)
            .field("vfs", &self.vfs)
            .finish()
    }
}

impl Deref for App {
    type Target = AppInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Inner type of [`App`] object.
pub struct AppInner {
    pub config: Arc<capwat_config::Server>,
    pub primary_db: PgPool,
    pub replica_db: Option<PgPool>,
    pub vfs: Vfs,

    pub jwt_encode_key: EncodingKey,
    pub jwt_decode_key: DecodingKey,
}
