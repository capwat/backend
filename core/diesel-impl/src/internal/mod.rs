use cfg_if::cfg_if;
use diesel::{ConnectionError, ConnectionResult};
use diesel_async::AsyncPgConnection;
use futures::{future::BoxFuture, FutureExt};
use std::sync::Arc;

mod wrapper;
pub use wrapper::AsyncConnectionWrapper;

pub type Pool =
    diesel_async::pooled_connection::deadpool::Pool<AsyncPgConnection>;

pub type PgConnection = AsyncPgConnection;
pub type PooledConn =
    diesel_async::pooled_connection::deadpool::Object<PgConnection>;

cfg_if! {
    if #[cfg(any(
        not(any(feature = "tls-accept-native-certs", feature = "tls-accept-any-certs")),
        all(feature = "tls-accept-native-certs", feature = "tls-accept-any-certs")
    ))] {
        compile_error!("Please enable either `tls-accept-native-certs` or `tls-accept-any-certs` feature. Refer to the documentation for its terms.");

        #[must_use]
        pub(super) fn make_rustls_cfg() -> rustls::ClientConfig {
            unreachable!()
        }
    }
}

#[cfg(feature = "tls-accept-any-certs")]
#[must_use]
pub(super) fn make_rustls_cfg() -> rustls::ClientConfig {
    #[derive(Debug)]
    struct NoCertVerifier;

    impl rustls::client::danger::ServerCertVerifier for NoCertVerifier {
        fn verify_server_cert(
            &self,
            _end_entity: &rustls::pki_types::CertificateDer<'_>,
            _intermediates: &[rustls::pki_types::CertificateDer<'_>],
            _server_name: &rustls::pki_types::ServerName<'_>,
            _ocsp_response: &[u8],
            _now: rustls::pki_types::UnixTime,
        ) -> std::result::Result<
            rustls::client::danger::ServerCertVerified,
            rustls::Error,
        > {
            Ok(rustls::client::danger::ServerCertVerified::assertion())
        }

        fn verify_tls12_signature(
            &self,
            _message: &[u8],
            _cert: &rustls::pki_types::CertificateDer<'_>,
            _dss: &rustls::DigitallySignedStruct,
        ) -> std::result::Result<
            rustls::client::danger::HandshakeSignatureValid,
            rustls::Error,
        > {
            Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
        }

        fn verify_tls13_signature(
            &self,
            _message: &[u8],
            _cert: &rustls::pki_types::CertificateDer<'_>,
            _dss: &rustls::DigitallySignedStruct,
        ) -> std::result::Result<
            rustls::client::danger::HandshakeSignatureValid,
            rustls::Error,
        > {
            Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
        }

        fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
            vec![]
        }
    }

    rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(std::sync::Arc::new(NoCertVerifier))
        .with_no_client_auth()
}

#[cfg(feature = "tls-accept-native-certs")]
#[must_use]
pub(super) fn make_rustls_cfg() -> rustls::ClientConfig {
    let mut root_store = rustls::RootCertStore::empty();

    // Assuming everyone who hosts/works to the server are tech savy
    let native_certs = rustls_native_certs::load_native_certs().expect(
        "Could not load platform certificates. Diagnose the problem from your native certificate store",
    );

    for cert in native_certs {
        root_store.add(cert).expect("Failed to add platform certificate");
    }

    rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth()
}

pub(super) fn establish_tls_connection(
    url: &str,
) -> BoxFuture<'_, ConnectionResult<AsyncPgConnection>> {
    let fut = async {
        let rustls_cfg = make_rustls_cfg();
        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(rustls_cfg);

        let (client, conn) = tokio_postgres::connect(url, tls)
            .await
            .map_err(|e| ConnectionError::BadConnection(e.to_string()))?;

        let (tx, rx) = tokio::sync::broadcast::channel(1);
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
            tokio::select! {
                result = conn => if let Err(err) = result {
                    if let Err(err) = tx.send(Arc::new(err)) {
                        tracing::warn!("Failed to send shutdown message: {err}");
                    }
                },
                _ = shutdown_rx => {}
            }
        });

        AsyncPgConnection::try_from(client, Some(rx), Some(shutdown_tx)).await
    };
    fut.boxed()
}
