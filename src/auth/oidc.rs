use base64::{engine::general_purpose, Engine as _};
use openidconnect::{
    core::{CoreClient, CoreGenderClaim, CoreProviderMetadata, CoreResponseType},
    reqwest::{async_http_client, AsyncHttpClientError},
    AccessToken, AccessTokenHash, AdditionalClaims, AuthenticationFlow, AuthorizationCode,
    ClientId, ClientSecret, CsrfToken, DiscoveryError, IssuerUrl, Nonce, OAuth2TokenResponse,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, StandardClaims,
    TokenIntrospectionResponse, UserInfoClaims,
};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};
use time::OffsetDateTime;
use tracing::warn;
use url::Url;

use super::oauth::OAuthProviderMetadata;

#[derive(Clone)]
pub struct AuthConfig {
    pub issuer_url: IssuerUrl,
    pub redirect_url: RedirectUrl,

    pub client_id: ClientId,
    pub client_secret: Option<ClientSecret>,

    pub scopes: Vec<Scope>,
    pub required_groups: Vec<String>,
}

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone, Debug)]
pub struct AuthSession(String);

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthData {
    pub access_token: AccessToken,
    pub user: StandardClaims<CoreGenderClaim>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GroupClaim {
    groups: Vec<String>,
}

type RawAccessToken = String;
type UnixTimestamp = i64;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct AuthenticatedUser {
    pub expiry: UnixTimestamp,
    pub subject: String,
    pub username: String,
}

impl AuthenticatedUser {
    pub fn is_valid(&self) -> bool {
        OffsetDateTime::now_utc().unix_timestamp() < self.expiry
    }
}

#[derive(Clone)]
pub struct AuthClient {
    config: AuthConfig,
    client: CoreClient,

    state: Arc<Mutex<HashMap<AuthSession, (CsrfToken, Nonce, PkceCodeVerifier)>>>,
    introspection_cache: Arc<RwLock<HashMap<RawAccessToken, AuthenticatedUser>>>,
}

impl AuthClient {
    pub async fn new(config: AuthConfig) -> Result<Self, DiscoveryError<AsyncHttpClientError>> {
        let oauth_metadata =
            OAuthProviderMetadata::discover_async(&config.issuer_url, async_http_client).await?;
        let oidc_metadata =
            CoreProviderMetadata::discover_async(config.issuer_url.clone(), async_http_client)
                .await?;

        let client = CoreClient::from_provider_metadata(
            oidc_metadata,
            config.client_id.clone(),
            config.client_secret.clone(),
        )
        .set_introspection_uri(oauth_metadata.introspection_endpoint)
        .set_revocation_uri(oauth_metadata.revocation_endpoint)
        .set_redirect_uri(config.redirect_url.clone());

        Ok(Self {
            config,
            client,
            state: Default::default(),
            introspection_cache: Default::default(),
        })
    }

    pub fn create_session(&self) -> (AuthSession, Url) {
        let session = AuthSession::new_random();
        let (pkce_code_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (authorize_url, csrf_state, nonce) = self
            .client
            .authorize_url(
                AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scopes(self.config.scopes.iter().cloned())
            .set_pkce_challenge(pkce_code_challenge)
            .url();

        let mut state = self.state.lock().expect("auth mutex poisoned");
        state.insert(session.clone(), (csrf_state, nonce, pkce_verifier));

        (session, authorize_url)
    }

    pub async fn authenticate(
        &self,
        session: AuthSession,
        code: AuthorizationCode,
        csrf_state: CsrfToken,
    ) -> Option<AuthData> {
        let (expected_csrf_state, nonce, pkce_verifier) = self
            .state
            .lock()
            .expect("auth mutex poisoned")
            .remove(&session)?;

        if csrf_state.secret() != expected_csrf_state.secret() {
            warn!("Authentication failed, CSRF state mismatch");
            return None;
        }

        let response = self
            .client
            .exchange_code(code)
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await;

        let tokens = match response {
            Ok(tokens) => tokens,
            Err(err) => {
                warn!("Authentication failed, could not exchange code: {err}");
                return None;
            }
        };

        let id_token = match tokens.extra_fields().id_token() {
            Some(token) => token,
            None => {
                warn!("Authentication failed, did not receive ID token");
                return None;
            }
        };

        let verifier = self.client.id_token_verifier();
        let id_claims = match id_token.claims(&verifier, &nonce) {
            Ok(claims) => claims,
            Err(err) => {
                warn!("Authentication failed, ID token verification failed: {err}");
                return None;
            }
        };

        // Verify the access token hash to ensure that the token hasn't been substituted
        if let Some(expected_hash) = id_claims.access_token_hash() {
            let signing_alg = match id_token.signing_alg() {
                Ok(alg) => alg,
                Err(err) => {
                    warn!("Authentication failed, ID token not signed but access token hash present: {err}");
                    return None;
                }
            };

            let actual_hash = match AccessTokenHash::from_token(tokens.access_token(), &signing_alg)
            {
                Ok(hash) => hash,
                Err(err) => {
                    warn!("Authentication failed, unable to hash access token: {err}");
                    return None;
                }
            };

            if *expected_hash != actual_hash {
                warn!("Authentication failed, access token hash does not match");
                return None;
            }
        }

        let user_info_req = match self.client.user_info(
            tokens.access_token().clone(),
            Some(id_claims.subject().clone()),
        ) {
            Ok(req) => req,
            Err(err) => {
                warn!("Authentication failed, unable to build user info request: {err}");
                return None;
            }
        };

        let user_info: UserInfoClaims<GroupClaim, CoreGenderClaim> =
            match user_info_req.request_async(async_http_client).await {
                Ok(user_info) => user_info,
                Err(err) => {
                    warn!("Authentication failed, failed to fetch user info: {err}");
                    return None;
                }
            };

        let missing_group = self
            .config
            .required_groups
            .iter()
            .find(|group| !user_info.additional_claims().groups.contains(group));

        if let Some(group) = missing_group {
            warn!("Authentication failed, user does not have required group: {group}");
            return None;
        }

        Some(AuthData {
            access_token: tokens.access_token().clone(),
            user: user_info.standard_claims().clone(),
        })
    }

    pub async fn introspect(&self, token: &AccessToken) -> Option<AuthenticatedUser> {
        if let Some(data) = self
            .introspection_cache
            .read()
            .expect("Authentication expiry cache poisoned")
            .get(token.secret())
        {
            if data.is_valid() {
                return Some(data.clone());
            }
        }

        self
            .client
            .introspect(token)
            .expect("Authentication endpoint does not support access token introspection which is required!")
            .request_async(async_http_client)
            .await
            .map(|r| {
                let mut cache = self.introspection_cache.write().expect("Authentication expiry cache poisoned"); 

                if r.active() {
                    if let (Some(subject), Some(username)) = (r.sub().map(ToString::to_string), r.username().map(ToString::to_string)) {
                        cache.insert(token.secret().clone(), AuthenticatedUser { expiry: r.exp().unwrap().timestamp(), subject, username });
                    } else {
                        warn!("Introspection failed, returned data does not contain subject and/or username");
                    }
                }

                cache.get(token.secret()).cloned()
            })
            .unwrap_or_default()
    }
}

impl AuthSession {
    fn new_random() -> Self {
        let random_bytes: Vec<u8> = (0..16).map(|_| thread_rng().gen::<u8>()).collect();
        Self(general_purpose::URL_SAFE_NO_PAD.encode(&random_bytes))
    }
}

impl AdditionalClaims for GroupClaim {}
