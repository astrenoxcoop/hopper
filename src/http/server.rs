use std::convert::Infallible;
use std::time::Duration;

use axum::{body::Body, extract::Request, http::HeaderValue, response::Response, routing::get, Router};
use axum_htmx::AutoVaryLayer;
use http::{
    header::{ACCEPT, ACCEPT_LANGUAGE},
    Method,
};
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::http::{
    context::WebContext, handle_index::handle_index, handle_policy::handle_policy,
    handle_spec::handle_spec,
};

pub fn build_router(web_context: WebContext) -> Router {
    let serve_dir = tower::service_fn(|_request: Request| async {
        Ok::<_, Infallible>(Response::new(Body::empty()))
    });

    Router::new()
        .route("/", get(handle_index))
        .route("/spec", get(handle_spec))
        .route("/policy", get(handle_policy))
        .nest_service("/static", serve_dir.clone())
        .fallback_service(serve_dir)
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::new(Duration::from_secs(10)),
        ))
        .layer(
            CorsLayer::new()
                .allow_origin(web_context.external_base.parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET])
                .allow_headers([ACCEPT_LANGUAGE, ACCEPT]),
        )
        .layer(AutoVaryLayer)
        .with_state(web_context.clone())
}
