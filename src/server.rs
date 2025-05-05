use axum::{Router, routing::post};

use crate::{database::establish_connection, routes::register::register};

pub enum RoutesPathes {
    UserRegister,
}

impl RoutesPathes {
    pub fn as_str(&self) -> &'static str {
        match self {
            RoutesPathes::UserRegister => "/user/register",
        }
    }
}

pub fn create_server() -> Router {
    let pool = establish_connection();

    Router::new()
        .route(RoutesPathes::UserRegister.as_str(), post(register))
        .with_state(pool)
}
