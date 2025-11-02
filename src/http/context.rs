use axum::extract::FromRef;
use axum_template::engine::Engine;
use minijinja::Environment;
use moka::future::Cache;
use std::{ops::Deref, sync::Arc};

use crate::cache::{ResolveAtUriResult, ResolveWebHostMetaResult};

pub type AppEngine = Engine<Environment<'static>>;

pub struct InnerWebContext {
    pub(crate) external_base: String,
    pub(crate) engine: AppEngine,
    pub(crate) http_client: reqwest::Client,
    pub(crate) resolve_webfinger_cache: Cache<String, ResolveWebHostMetaResult>,
    pub(crate) resolve_aturi_cache: Cache<String, ResolveAtUriResult>,
}

#[derive(Clone, FromRef)]
pub struct WebContext(pub(crate) Arc<InnerWebContext>);

impl Deref for WebContext {
    type Target = InnerWebContext;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl WebContext {
    pub fn new(
        external_base: &str,
        engine: AppEngine,
        http_client: &reqwest::Client,
        resolve_webfinger_cache: Cache<String, ResolveWebHostMetaResult>,
        resolve_aturi_cache: Cache<String, ResolveAtUriResult>,
    ) -> Self {
        Self(Arc::new(InnerWebContext {
            external_base: external_base.to_string(),
            engine,
            http_client: http_client.clone(),
            resolve_webfinger_cache,
            resolve_aturi_cache,
        }))
    }
}
