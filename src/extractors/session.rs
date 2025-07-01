use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use axum_extra::extract::CookieJar;
use jsonwebtoken::TokenData;
use serde::{Deserialize, Serialize};

use crate::jwt::{decode_jwt, encode_jwt};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSessionClaims {
    pub exp: usize,
    pub id: i32,
    pub email: String,
    pub display_name: String,
}

impl<S> FromRequestParts<S> for UserSessionClaims
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(session_token) = CookieJar::from_headers(&parts.headers).get("session_token") {
            Ok(decode_jwt_user_session(&session_token.value().to_string()).claims)
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

pub fn encode_jwt_user_session(email: String, display_name: String, id: i32) -> String {
    encode_jwt(&UserSessionClaims {
        exp: usize::try_from(
            time::UtcDateTime::now().unix_timestamp()
                + i64::try_from(time::Duration::days(30).whole_milliseconds()).unwrap(),
        )
        .unwrap(),
        email: email,
        id: id,
        display_name: display_name,
    })
}

pub fn decode_jwt_user_session(session_token: &String) -> TokenData<UserSessionClaims> {
    decode_jwt::<UserSessionClaims>(session_token)
}
