mod any;
pub use self::any::AnyPool;

use diesel::{ConnectionError, ConnectionResult};
use diesel_async::AsyncPgConnection;
use futures::{future::BoxFuture, FutureExt};

/// Establishes PostgreSQL connection with TLS encryption tunnel
/// enabled without checking if TLS is needed to connect.
pub fn establish_connection_with_tls(
    config: &str,
) -> BoxFuture<ConnectionResult<AsyncPgConnection>> {
    let fut = async {
        let rustls_cfg = rustls::ClientConfig::builder()
            .with_root_certificates(get_root_certs())
            .with_no_client_auth();

        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(rustls_cfg);
        let (client, conn) = tokio_postgres::connect(config, tls)
            .await
            .map_err(|e| ConnectionError::BadConnection(e.to_string()))?;

        AsyncPgConnection::try_from_client_and_connection(client, conn).await
    };
    fut.boxed()
}

fn get_root_certs() -> rustls::RootCertStore {
    let mut roots = rustls::RootCertStore::empty();
    let certs = rustls_native_certs::load_native_certs().expect("could not load root certs!");
    roots.add_parsable_certificates(certs);
    roots
}
