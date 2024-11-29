use axum::{middleware::from_fn, Router};
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::compression::CompressionLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::request_id::PropagateRequestIdLayer;
use tower_http::timeout::{RequestBodyTimeoutLayer, TimeoutLayer};

pub mod panic;
pub mod telemetry;

const MAX_CONTENT_LEN: usize = 10 * 10244; // 10 KB

pub fn apply(router: Router) -> Router {
    let middleware = ServiceBuilder::new()
        .layer(self::telemetry::set_request_id_layer())
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(from_fn(self::telemetry::trace_request))
        .layer(CatchPanicLayer::custom(self::panic::catch_panic));

    // being too generous to support various compression algorithms
    // because no one agrees which one is the best
    let compression_layer = CompressionLayer::new()
        .br(true)
        .zstd(true)
        .gzip(true)
        .deflate(true)
        .quality(tower_http::CompressionLevel::Fastest);

    router
        .layer(middleware)
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(RequestBodyTimeoutLayer::new(Duration::from_secs(30)))
        .layer(RequestBodyLimitLayer::new(MAX_CONTENT_LEN))
        .layer(compression_layer)
}
