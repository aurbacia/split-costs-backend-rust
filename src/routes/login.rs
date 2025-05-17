use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordVerifier},
};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use deadpool_diesel::postgres::Pool;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

use axum_extra::extract::cookie::{Cookie, CookieJar};

use crate::{jwt::encode_jwt, models::User, validate::ValidatedPayload};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSessionClaims {
    pub exp: usize,
    pub email: String,
    pub display_name: String,
}

fn encode_jwt_user_session(email: String, display_name: String) -> String {
    encode_jwt(&UserSessionClaims {
        exp: usize::try_from(
            time::UtcDateTime::now().unix_timestamp()
                + i64::try_from(time::Duration::days(30).whole_milliseconds()).unwrap(),
        )
        .unwrap(),
        email: email,
        display_name: display_name,
    })
}

pub async fn login(
    jar: CookieJar,
    State(pool): State<Pool>,
    ValidatedPayload(payload): ValidatedPayload<Payload>,
) -> impl IntoResponse {
    use crate::schema::users::dsl::*;

    let conn = pool.get().await.expect("Can't get connection from pool");
    match conn
        .interact(move |conn| {
            users
                .select(User::as_select())
                .filter(email.eq(&payload.email))
                .get_result(conn)
                .optional()
                .unwrap()
        })
        .await
    {
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(None) => StatusCode::UNAUTHORIZED.into_response(),
        Ok(user) => {
            let user_info = user.unwrap();
            match Argon2::default().verify_password(
                payload.password.as_ref(),
                &PasswordHash::new(&user_info.password).unwrap(),
            ) {
                Err(_) => (StatusCode::UNAUTHORIZED).into_response(),
                Ok(_) => {
                    let token = encode_jwt_user_session(user_info.email, user_info.display_name);

                    jar.add(Cookie::new("session_token", token.clone()))
                        .into_response()
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Validate)]
pub struct Payload {
    #[validate(length(min = 10))]
    pub password: String,

    #[validate(email)]
    pub email: String,
}

#[derive(Serialize)]
pub struct ResultBody {
    pub token: String,
}
