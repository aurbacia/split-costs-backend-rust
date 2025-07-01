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

use crate::{
    extractors::session::encode_jwt_user_session, models::User, validate::ValidatedPayload,
};

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
                    let token = encode_jwt_user_session(
                        user_info.email,
                        user_info.display_name,
                        user_info.id,
                    );

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
