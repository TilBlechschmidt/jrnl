use axum::{
    async_trait,
    body::Body,
    extract::{FromRequestParts, Query},
    http::{header::REFERER, request::Parts, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Extension, Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use openidconnect::{AccessToken, AuthorizationCode, CsrfToken};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use time::Duration;

pub mod oauth;
pub mod oidc;
pub use oidc::AuthenticatedUser;

// This can be changed for local development but really should be true in production
const REQUIRE_HTTPS: bool = false;
const AUTH_COOKIE: &'static str = "auth";
const USER_COOKIE: &'static str = "user";
const REDIRECT_COOKIE: &'static str = "redirectURL";

#[derive(Deserialize)]
pub struct CallbackData {
    code: AuthorizationCode,
    state: CsrfToken,
}

pub fn router() -> Router<(), Body> {
    Router::<(), Body>::new()
        .route("/login", get(login))
        .route("/callback", get(callback))
        .route("/success", get(success))
        .route("/failed", get(failed))
}

async fn login(
    mut jar: CookieJar,
    // TODO Use an extension instead!
    Extension(auth_client): Extension<oidc::AuthClient>,
    headers: HeaderMap,
) -> (CookieJar, Redirect) {
    let (auth_session, auth_url) = auth_client.create_session();

    if let Some(referrer) = headers.get(REFERER).map(|h| h.to_str().ok()).flatten() {
        jar = jar.add(
            Cookie::build(REDIRECT_COOKIE, referrer.to_owned())
                .secure(REQUIRE_HTTPS)
                .max_age(Duration::MINUTE * 5)
                .same_site(SameSite::Lax)
                .http_only(true)
                .path("/")
                .finish(),
        );
    }

    (
        AuthState::Pending(auth_session).write_to_jar(jar),
        Redirect::to(auth_url.as_str()),
    )
}

async fn callback(
    Query(data): Query<CallbackData>,
    jar: CookieJar,
    Extension(auth_client): Extension<oidc::AuthClient>,
) -> (CookieJar, Redirect) {
    if let AuthState::Pending(session) = AuthState::from_jar(&jar) {
        if let Some(auth) = auth_client
            .authenticate(session, data.code, data.state)
            .await
        {
            let user_cookie = build_user_cookie(&auth);
            return (
                AuthState::Authenticated(auth.access_token)
                    .write_to_jar(jar)
                    .add(user_cookie),
                Redirect::to("./success"),
            );
        }
    };

    (jar, Redirect::to("./failed"))
}

async fn success(jar: CookieJar) -> Response {
    if let Some(destination) = jar.get(REDIRECT_COOKIE).cloned() {
        (
            jar.remove(Cookie::named(REDIRECT_COOKIE)),
            Html(format!(
                r#"
                    Login successful.
                    <script>window.location = {};</script>
                "#,
                serde_json::to_string(destination.value()).unwrap_or_default()
            )),
        )
            .into_response()
    } else {
        "Login successful.".into_response()
    }
}

async fn failed() -> (StatusCode, &'static str) {
    (
        StatusCode::UNAUTHORIZED,
        "Login failed. See server logs for more details.",
    )
}

fn build_user_cookie(data: &oidc::AuthData) -> Cookie<'static> {
    Cookie::build(
        USER_COOKIE,
        serde_json::to_string(&data.user).expect("failed to serialize user cookie"),
    )
    .secure(REQUIRE_HTTPS)
    .max_age(Duration::WEEK)
    .same_site(SameSite::Strict)
    .path("/")
    .finish()
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AuthState {
    Pending(oidc::AuthSession),
    Authenticated(AccessToken),

    #[serde(other)]
    Unauthenticated,
}

impl AuthState {
    fn from_jar(jar: &CookieJar) -> AuthState {
        jar.get(AUTH_COOKIE)
            .map(|cookie| serde_json::from_str(cookie.value()).ok())
            .flatten()
            .unwrap_or(AuthState::Unauthenticated)
    }

    fn write_to_jar(&self, jar: CookieJar) -> CookieJar {
        // For the callback to work the pending cookie has to be set as lax
        let same_site = match &self {
            AuthState::Pending(_) => SameSite::Lax,
            _ => SameSite::Strict,
        };

        let value = serde_json::to_string(&self).expect("failed to serialize AuthState");
        let cookie = Cookie::build(AUTH_COOKIE, value)
            .secure(REQUIRE_HTTPS)
            .http_only(true)
            .max_age(self.validity_period())
            .same_site(same_site)
            .path("/")
            .finish();

        jar.add(cookie)
    }

    fn validity_period(&self) -> Duration {
        match self {
            AuthState::Pending(_) => Duration::MINUTE * 5,
            AuthState::Authenticated(_) => Duration::DAY,
            AuthState::Unauthenticated => Duration::ZERO,
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthState
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_client = parts
            .extensions
            .get::<oidc::AuthClient>()
            .expect("missing AuthClient extension");

        let jar = CookieJar::from_headers(&parts.headers);
        let state = Self::from_jar(&jar);

        Ok(if let AuthState::Authenticated(token) = &state {
            // Make sure the token is still valid!
            if auth_client.introspect(token).await.is_none() {
                AuthState::Unauthenticated
            } else {
                state
            }
        } else {
            state
        })
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Html<&'static str>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        const UNAUTHORIZED: (StatusCode, Html<&'static str>) = (
            StatusCode::UNAUTHORIZED,
            Html("Unauthorized. <a href=\"/auth/login\">Login -></a>"),
        );

        if let AuthState::Authenticated(token) = AuthState::from_request_parts(parts, state)
            .await
            .map_err(|_| UNAUTHORIZED)?
        {
            parts
                .extensions
                .get::<oidc::AuthClient>()
                .expect("missing AuthClient extension")
                .introspect(&token)
                .await
                .ok_or(UNAUTHORIZED)
        } else {
            Err(UNAUTHORIZED)
        }
    }
}
