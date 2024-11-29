use axum::extract::{MatchedPath, Request};
use axum::middleware::Next;
use axum::response::Response;
use axum::Extension;
use axum_extra::headers::{Header, UserAgent};
use axum_extra::TypedHeader;
use chrono::Utc;
use std::time::Instant;
use tower_http::request_id::{MakeRequestId, RequestId, SetRequestIdLayer};
use tracing::{debug, Instrument};
use uuid::Uuid;

use crate::headers::XRequestId;

#[doc(hidden)]
#[derive(Default, Clone)]
pub struct LocalRequestIdGenerator;

impl MakeRequestId for LocalRequestIdGenerator {
    fn make_request_id<B>(
        &mut self,
        _request: &axum::http::Request<B>,
    ) -> Option<tower_http::request_id::RequestId> {
        Uuid::now_v7().to_string().parse().ok().map(RequestId::new)
    }
}

#[must_use]
pub fn set_request_id_layer() -> SetRequestIdLayer<LocalRequestIdGenerator> {
    SetRequestIdLayer::new(XRequestId::name().clone(), LocalRequestIdGenerator)
}

#[doc(hidden)]
#[derive(axum::extract::FromRequestParts)]
pub struct ExtraMetadata {
    path: Option<Extension<MatchedPath>>,
    request_id: Option<TypedHeader<XRequestId>>,
    user_agent: Option<TypedHeader<UserAgent>>,
}

// TODO: Document everything here for anti-telemetry freaks
pub async fn trace_request(metadata: ExtraMetadata, request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let timestamp = Utc::now();
    let target = metadata
        .path
        .as_ref()
        .map(|p| p.as_str())
        .unwrap_or_else(|| request.uri().path())
        .to_string();

    let version = format!("{:?}", request.version());
    let span = tracing::info_span!(
        "http.request",
        http.method = %method,
        http.status_code = tracing::field::Empty,
        http.target = %target,
        http.user_agent = %metadata.user_agent.as_ref().map(|v| v.as_str()).unwrap_or_default(),
        http.version = %version,
        request.duration = tracing::field::Empty,
        request.id = %metadata.request_id.as_ref().map(|v| v.as_str()).unwrap_or_default(),
        request.timestamp = %timestamp,
    );

    span.in_scope(|| debug!("Processing request: {target}"));

    let start = Instant::now();
    let response = next.run(request).instrument(span.clone()).await;
    let elapsed = start.elapsed();

    let status = response.status();
    span.record("http.status_code", tracing::field::display(status.as_u16()));
    span.record("request.duration", tracing::field::debug(elapsed));
    span.in_scope(|| debug!("{method} {target:?} {version} -> {status} ({elapsed:?})"));

    response
}
