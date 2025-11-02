use anyhow::Result;
use hopper::{
    cache::{new_resolve_aturi_cache, new_resolve_webhostmeta_cache, ResolveWebHostMetaResult},
    http::{
        context::{AppEngine, WebContext},
        server::build_router,
        templates,
    },
    webhostmeta::WebHostMeta,
};
use std::{env, time::Duration};
use tokio::net::TcpListener;
use tokio::signal;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "hopper=debug,info".into()),
        ))
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    let version = hopper::config::version()?;

    env::args().for_each(|arg| {
        if arg == "--version" {
            println!("{}", version);
            std::process::exit(0);
        }
    });

    let config = hopper::config::Config::new()?;

    let mut client_builder = reqwest::Client::builder();
    for ca_certificate in config.certificate_bundles.as_ref() {
        tracing::info!("Loading CA certificate: {:?}", ca_certificate);
        let cert = std::fs::read(ca_certificate)?;
        let cert = reqwest::Certificate::from_pem(&cert)?;
        client_builder = client_builder.add_root_certificate(cert);
    }

    client_builder = client_builder.user_agent(config.user_agent.clone());
    client_builder = client_builder.read_timeout(Duration::from_secs(1));
    client_builder = client_builder.connect_timeout(Duration::from_secs(1));
    client_builder = client_builder.timeout(Duration::from_secs(3));
    let http_client = client_builder.build()?;

    let jinja = templates::build_env(config.external_base.clone(), config.version.clone());

    let resolve_webfinger_cache = new_resolve_webhostmeta_cache();

    resolve_webfinger_cache
        .insert(
            "bsky.app".to_string(),
            ResolveWebHostMetaResult::Found(WebHostMeta::new(vec![
                hopper::webhostmeta::Link::new("https://bsky.app/profile/{authority}", None),
                hopper::webhostmeta::Link::new(
                    "https://bsky.app/profile/{authority}/post/{rkey}",
                    Some("app.bsky.feed.post"),
                ),
            ])),
        )
        .await;

    resolve_webfinger_cache
        .insert(
            "frontpage.fyi".to_string(),
            ResolveWebHostMetaResult::Found(WebHostMeta::new(vec![
                hopper::webhostmeta::Link::new(
                    "https://frontpage.fyi/post/{authority}/{rkey}",
                    Some("fyi.unravel.frontpage.post"),
                ),
            ])),
        )
        .await;

        resolve_webfinger_cache
        .insert(
            "whtwnd.com".to_string(),
            ResolveWebHostMetaResult::Found(WebHostMeta::new(vec![
                hopper::webhostmeta::Link::new(
                    "https://whtwnd.com/{authority}/{rkey}",
                    Some("com.whtwnd.blog.entry"),
                ),
            ])),
        )
        .await;

    let resolve_aturi_cache = new_resolve_aturi_cache();

    let web_context = WebContext::new(
        config.external_base.as_str(),
        AppEngine::from(jinja),
        &http_client,
        resolve_webfinger_cache,
        resolve_aturi_cache,
    );

    let app = build_router(web_context.clone());

    let tracker = TaskTracker::new();
    let token = CancellationToken::new();

    {
        let tracker = tracker.clone();
        let inner_token = token.clone();

        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        tokio::spawn(async move {
            tokio::select! {
                () = inner_token.cancelled() => { },
                _ = terminate => {},
                _ = ctrl_c => {},
            }

            tracker.close();
            inner_token.cancel();
        });
    }

    {
        let inner_config = config.clone();
        let http_port = *inner_config.http_port.as_ref();
        let inner_token = token.clone();
        tracker.spawn(async move {
            let listener = TcpListener::bind(&format!("0.0.0.0:{}", http_port))
                .await
                .unwrap();

            let shutdown_token = inner_token.clone();
            let result = axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    tokio::select! {
                        () = shutdown_token.cancelled() => { }
                    }
                    tracing::info!("axum graceful shutdown complete");
                })
                .await;
            if let Err(err) = result {
                tracing::error!("axum task failed: {}", err);
            }

            inner_token.cancel();
        });
    }

    tracker.wait().await;

    Ok(())
}
