use axum::extract::State;
use axum::{http::StatusCode, response::IntoResponse};

use deadpool_diesel::postgres::Pool;
use diesel::dsl::exists;
use diesel::{insert_into, prelude::*, select};
use serde::{Deserialize, Serialize};
use validator::Validate;

use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};

use crate::validate::ValidatedPayload;

fn hash_password(password: &[u8]) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password, &salt)
        .unwrap()
        .to_owned()
        .to_string()
}

pub async fn register(
    State(pool): State<Pool>,
    ValidatedPayload(payload): ValidatedPayload<Payload>,
) -> impl IntoResponse {
    use crate::schema::users::dsl::*;

    let conn = pool.get().await.expect("Can't get connection from pool");
    match conn
        .interact(move |conn| {
            select(exists(users.filter(email.eq(&payload.email))))
                .get_result::<bool>(conn)
                .is_ok_and(|r| r == false)
                .then(|| {
                    insert_into(users)
                        .values(Payload {
                            password: hash_password(payload.password.as_ref()),
                            ..payload.clone()
                        })
                        .execute(conn)
                        .unwrap()
                })
        })
        .await
    {
        Ok(None) => StatusCode::UNPROCESSABLE_ENTITY,
        Ok(_) => StatusCode::CREATED,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[derive(Serialize, Deserialize, Insertable, Clone, Default, Validate)]
#[diesel(table_name = crate::schema::users)]
pub struct Payload {
    #[validate(length(min = 3))]
    pub display_name: String,

    #[validate(length(min = 10))]
    pub password: String,

    #[validate(email)]
    pub email: String,
}
