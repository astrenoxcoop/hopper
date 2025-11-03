use anyhow::Result;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use axum_template::RenderHtml;
use minijinja::context as template_context;
use ordermap::OrderSet;
use serde::Deserialize;

use crate::{
    cache::aturi_cached,
    errors::HopperError,
    http::context::WebContext,
    model::validate_aturi,
};

pub(crate) const ERROR_INVALID_AT_URI: &str = "Invalid AT-URI";

#[derive(Deserialize)]
pub(crate) struct Destination {
    aturi: Option<String>,
    server: Option<String>,
}

pub(crate) async fn handle_index(
    State(web_context): State<WebContext>,
    Query(destination): Query<Destination>,
) -> Result<impl IntoResponse, HopperError> {
    let default_context = template_context! {
        canonical_url => format!("https://{}/", web_context.external_base),
    };

    if let Some(aturi_str) = destination.aturi {
        let aturi = validate_aturi(&aturi_str);
        if aturi.is_none() {
            tracing::debug!(error = ERROR_INVALID_AT_URI, "error encountered");

            return Ok(RenderHtml(
                "index.html",
                web_context.engine.clone(),
                template_context! { ..default_context, ..template_context! {
                    handle_error => true,
                    aturi_value => aturi_str,
                    aturi_error => ERROR_INVALID_AT_URI,
                }},
            )
            .into_response());
        }

        let aturi = aturi.unwrap();

        let servers = parse_servers(&destination.server.unwrap_or_default());

        let destination = aturi_cached(
            &web_context.http_client,
            &web_context.resolve_webfinger_cache,
            &web_context.resolve_aturi_cache,
            &servers,
            &aturi_str,
            &aturi,
        )
        .await;

        if let Err(err) = destination {
            tracing::debug!(error = ?err, "error encountered");
            let error_message = err.to_string();

            return Ok(RenderHtml(
                "index.html",
                web_context.engine.clone(),
                template_context! { ..default_context, ..template_context! {
                    handle_error => true,
                    aturi_value => aturi_str,
                    aturi_error => error_message,
                }},
            )
            .into_response());
        }

        let destination = destination.unwrap();

        return Ok(Redirect::to(&destination).into_response());
    }

    Ok(RenderHtml(
        "index.html",
        web_context.engine.clone(),
        default_context,
    )
    .into_response())
}

fn parse_servers(value: &str) -> Vec<String> {
    let mut values = value
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<OrderSet<String>>();

    values.extend(vec![
        "smokesignal.events".into(),
        "frontpage.fyi".into(),
        "whtwnd.com".into(),
        "bsky.app".into(),
    ]);

    Vec::from_iter(values)
}
