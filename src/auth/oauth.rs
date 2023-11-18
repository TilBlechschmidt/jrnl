use axum::http::{header::ACCEPT, HeaderValue, Method, StatusCode};
use openidconnect::{
    DiscoveryError, HttpRequest, HttpResponse, IntrospectionUrl, IssuerUrl, RevocationUrl,
};
use serde::Deserialize;
use std::future::Future;

const OAUTH_CONFIG_URL_SUFFIX: &'static str = ".well-known/oauth-authorization-server";

#[derive(Debug, Deserialize)]
pub struct OAuthProviderMetadata {
    issuer: IssuerUrl,

    pub introspection_endpoint: IntrospectionUrl,
    pub revocation_endpoint: RevocationUrl,
}

impl OAuthProviderMetadata {
    #[allow(dead_code)]
    pub fn discover<HC, RE>(
        issuer_url: &IssuerUrl,
        http_client: HC,
    ) -> Result<Self, DiscoveryError<RE>>
    where
        HC: Fn(HttpRequest) -> Result<HttpResponse, RE>,
        RE: std::error::Error + 'static,
    {
        let discovery_url = issuer_url
            .join(OAUTH_CONFIG_URL_SUFFIX)
            .map_err(DiscoveryError::UrlParse)?;

        http_client(Self::discovery_request(discovery_url))
            .map_err(DiscoveryError::Request)
            .and_then(|http_response| Self::discovery_response(issuer_url, http_response))
    }

    pub async fn discover_async<F, HC, RE>(
        issuer_url: &IssuerUrl,
        http_client: HC,
    ) -> Result<Self, DiscoveryError<RE>>
    where
        F: Future<Output = Result<HttpResponse, RE>>,
        HC: Fn(HttpRequest) -> F,
        RE: std::error::Error + 'static,
    {
        let discovery_url = issuer_url
            .join(OAUTH_CONFIG_URL_SUFFIX)
            .map_err(DiscoveryError::UrlParse)?;

        http_client(Self::discovery_request(discovery_url))
            .await
            .map_err(DiscoveryError::Request)
            .and_then(|http_response| Self::discovery_response(issuer_url, http_response))
    }

    fn discovery_request(discovery_url: url::Url) -> HttpRequest {
        HttpRequest {
            url: discovery_url,
            method: Method::GET,
            headers: vec![(ACCEPT, HeaderValue::from_static("application/json"))]
                .into_iter()
                .collect(),
            body: Vec::new(),
        }
    }

    fn discovery_response<RE>(
        issuer_url: &IssuerUrl,
        discovery_response: HttpResponse,
    ) -> Result<Self, DiscoveryError<RE>>
    where
        RE: std::error::Error + 'static,
    {
        if discovery_response.status_code != StatusCode::OK {
            return Err(DiscoveryError::Response(
                discovery_response.status_code,
                discovery_response.body,
                format!("HTTP status code {}", discovery_response.status_code),
            ));
        }

        let provider_metadata = serde_path_to_error::deserialize::<_, Self>(
            &mut serde_json::Deserializer::from_slice(&discovery_response.body),
        )
        .map_err(DiscoveryError::Parse)?;

        if provider_metadata.issuer != *issuer_url {
            Err(DiscoveryError::Validation(format!(
                "unexpected issuer URI `{}` (expected `{}`)",
                provider_metadata.issuer.as_str(),
                issuer_url.as_str()
            )))
        } else {
            Ok(provider_metadata)
        }
    }
}
