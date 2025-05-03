use axum::Json;
use axum::extract::State;
use axum::{http::StatusCode, response::IntoResponse};

use deadpool_diesel::postgres::Pool;
use diesel::dsl::exists;
use diesel::{insert_into, prelude::*, select};
use serde::Deserialize;

use crate::schema::users::dsl::*;

pub async fn register(State(pool): State<Pool>, Json(payload): Json<Payload>) -> impl IntoResponse {
    let conn = pool.get().await.expect("Can't get connection from pool");

    match conn
        .interact(|conn| {
            select(exists(users.filter(email.eq(&payload.email))))
                .get_result::<bool>(conn)
                .is_ok_and(|r| r == false)
                .then(|| insert_into(users).values(payload).execute(conn).unwrap())
        })
        .await
    {
        Ok(None) => StatusCode::UNPROCESSABLE_ENTITY,
        Ok(_) => StatusCode::CREATED,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[derive(Deserialize, Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct Payload {
    login: String,
    display_name: String,
    password: String,
    email: String,
}
