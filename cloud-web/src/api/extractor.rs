use crate::api::error::CustomError;
use axum::extract::{Extension, FromRequestParts};

use async_trait::async_trait;
use crate::api::ApiContext;
use axum::http::header::AUTHORIZATION;
use axum::http::HeaderValue;
use axum::http::request::Parts;
use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use sha2::Sha384;
use time::OffsetDateTime;
use uuid::Uuid;
use crate::api::Result;

const DEFAULT_SESSION_LENGTH: time::Duration = time::Duration::weeks(1);

// Ideally the Realworld spec would use the `Bearer` scheme as that's relatively standard
// and has parsers available, but it's really not that hard to parse anyway.
const SCHEME_PREFIX: &str = "Token ";

/// Add this as a parameter to a handler function to require the user to be logged in.
///
/// Parses a JWT from the `Authorization: Token <token>` header.
///
pub struct AuthUser {
    pub user_id: Uuid,
}

/// Add this as a parameter to a handler function to optionally check if the user is logged in.
///
/// If the `Authorization` header is absent then this will be `Self(None)`, otherwise it will
/// validate the token.
///
/// This is in contrast to directly using `Option<AuthUser>`, which will be `None` if there
/// is *any* error in deserializing, which isn't exactly what we want.
pub struct MaybeAuthUser(pub Option<AuthUser>);

#[derive(serde::Serialize, serde::Deserialize)]
struct AuthUserClaims {
    user_id: Uuid,
    exp: i64,
}

type HmacSha384 = Hmac<Sha384>;

impl AuthUser {
    pub fn to_jwt(&self, ctx: &ApiContext) -> String {
        let hmac = HmacSha384::new_from_slice(ctx.config.hmac_key.as_bytes())
            .expect("HMAC-SHA-384 can accept any key length");

        AuthUserClaims {
            user_id: self.user_id,
            exp: (OffsetDateTime::now_utc() + DEFAULT_SESSION_LENGTH).unix_timestamp(),
        }
        .sign_with_key(&hmac)
        .expect("HMAC signing should be infallible")
    }

    /// Attempt to parse `Self` from an `Authorization` header.
    fn from_authorization(ctx: &ApiContext, auth_header: &HeaderValue) -> Result<Self> {
        let auth_header = auth_header.to_str().map_err(|_| {
            CustomError::Unauthorized
        })?;

        if !auth_header.starts_with(SCHEME_PREFIX) {
            return Err(CustomError::Unauthorized);
        }

        let token = &auth_header[SCHEME_PREFIX.len()..];

        let jwt =
            jwt::Token::<jwt::Header, AuthUserClaims, _>::parse_unverified(token).map_err(|e| {
                CustomError::Unauthorized
            })?;

        // Realworld doesn't specify the signing algorithm for use with the JWT tokens
        // so we picked SHA-384 (HS-384) as the HMAC, as it is more difficult to brute-force
        // than SHA-256 (recommended by the JWT spec) at the cost of a slightly larger token.
        let hmac = Hmac::<Sha384>::new_from_slice(ctx.config.hmac_key.as_bytes())
            .expect("HMAC-SHA-384 can accept any key length");

        // When choosing a JWT implementation, be sure to check that it validates that the signing
        // algorithm declared in the token matches the signing algorithm you're verifying with.
        // The `jwt` crate does.
        let jwt = jwt.verify_with_key(&hmac).map_err(|e| {
            CustomError::Unauthorized
        })?;

        let (_header, claims) = jwt.into();
        if claims.exp < OffsetDateTime::now_utc().unix_timestamp() {
            return Err(CustomError::Unauthorized);
        }

        Ok(Self {
            user_id: claims.user_id,
        })
    }
}

impl MaybeAuthUser {
    /// If this is `Self(Some(AuthUser))`, return `AuthUser::user_id`
    pub fn user_id(&self) -> Option<Uuid> {
        self.0.as_ref().map(|auth_user| auth_user.user_id)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser 
where
    S: Send + Sync,
{
    type Rejection = CustomError;

    async fn from_request_parts(req: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let ctx: Extension<ApiContext> = Extension::from_request_parts(req, _state)
            .await
            .expect("BUG: ApiContext was not added as an extension");

        // Get the value of the `Authorization` header, if it was sent at all.
        let auth_header = req.headers
            .get(AUTHORIZATION)
            .ok_or(CustomError::Unauthorized)?;

        Self::from_authorization(&ctx, auth_header)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for MaybeAuthUser
where 
    S: Send + Sync
{
    type Rejection = CustomError;

    async fn from_request_parts(req: &mut Parts, s: &S) -> Result<Self, Self::Rejection> {
        let ctx: Extension<ApiContext> = Extension::from_request_parts(req, s)
            .await
            .expect("BUG: ApiContext was not added as an extension");
        let auth_header = req.headers
            .get(AUTHORIZATION)
            .ok_or(CustomError::Unauthorized)?;

        //Ok(Some(AuthUser::from_authorization(&ctx, auth_header)))

        let user: Option<AuthUser> = AuthUser::from_authorization(&ctx, auth_header).map(Some).unwrap_or(None);
        Ok(Self(user))
    }
}
