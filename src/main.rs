#![feature(round_char_boundary)]

use axum::{Extension, Router};
use openidconnect::{ClientId, ClientSecret, IssuerUrl, RedirectUrl, Scope};
use std::{env, net::SocketAddr};

mod api;
mod auth;
mod frontend;
mod storage;

const ENV_STORAGE_LOCATION: &'static str = "THOUGHT_STORAGE_LOCATION";
const ENV_OIDC_ISSUER: &'static str = "THOUGHT_OIDC_ISSUER_URL";
const ENV_OIDC_REDIRECT_URL: &'static str = "THOUGHT_OIDC_REDIRECT_URL";
const ENV_OIDC_CLIENT_ID: &'static str = "THOUGHT_OIDC_CLIENT_ID";
const ENV_OIDC_CLIENT_SECRET: &'static str = "THOUGHT_OIDC_CLIENT_SECRET";
const ENV_OIDC_SCOPES: &'static str = "THOUGHT_OIDC_SCOPES";
const ENV_OIDC_GROUPS: &'static str = "THOUGHT_OIDC_GROUPS";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let issuer_url = IssuerUrl::new(
        env::var(ENV_OIDC_ISSUER).expect(&format!("env var {ENV_OIDC_ISSUER} not set")),
    )
    .expect("invalid oidc issuer url");

    let redirect_url = RedirectUrl::new(
        env::var(ENV_OIDC_REDIRECT_URL).expect(&format!("env var {ENV_OIDC_REDIRECT_URL} not set")),
    )
    .expect("invalid oidc redirect url");

    let client_id = ClientId::new(
        env::var(ENV_OIDC_CLIENT_ID).expect(&format!("env var {ENV_OIDC_CLIENT_ID} not set")),
    );

    let client_secret = Some(ClientSecret::new(
        env::var(ENV_OIDC_CLIENT_SECRET)
            .expect(&format!("env var {ENV_OIDC_CLIENT_SECRET} not set")),
    ));

    let scopes = env::var(ENV_OIDC_SCOPES)
        .unwrap_or_default()
        .split(" ")
        .filter(|s| !s.is_empty())
        .map(|s| Scope::new(s.to_owned()))
        .collect();

    let required_groups = env::var(ENV_OIDC_GROUPS)
        .unwrap_or_default()
        .split(" ")
        .filter(|s| !s.is_empty())
        .map(|s| s.to_owned())
        .collect();

    let auth_config = auth::oidc::AuthConfig {
        issuer_url,
        redirect_url,

        client_id,
        client_secret,

        scopes,

        required_groups,
    };

    let auth_client = auth::oidc::AuthClient::new(auth_config).await.unwrap();

    let app = Router::new()
        .nest("/auth", auth::router())
        .nest("/api", api::router())
        .fallback_service(frontend::service())
        .layer(Extension(auth_client));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
