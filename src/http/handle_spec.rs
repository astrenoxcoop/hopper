use anyhow::Result;
use axum::{extract::State, response::IntoResponse};
use axum_template::RenderHtml;
use minijinja::context as template_context;

use crate::{
    errors::HopperError,
    http::context::WebContext,
};

pub async fn handle_spec(
    State(web_context): State<WebContext>,
) -> Result<impl IntoResponse, HopperError> {
    let default_context = template_context! {
        canonical_url => format!("https://{}/spec", web_context.external_base),
    };

    Ok(RenderHtml(
        "spec.html",
        web_context.engine.clone(),
        default_context,
    )
    .into_response())
}
