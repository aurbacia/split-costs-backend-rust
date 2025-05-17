use axum::{Router, routing::post};

use crate::{
    database::establish_connection,
    routes::{login::login, register::register},
};

pub enum RoutesPathes {
    UserRegister,
    UserLogin,
}

impl RoutesPathes {
    pub fn as_str(&self) -> &'static str {
        match self {
            RoutesPathes::UserRegister => "/user/register",
            RoutesPathes::UserLogin => "/user/login",
        }
    }
}

pub fn create_server() -> Router {
    let pool = establish_connection();

    Router::new()
        .route(RoutesPathes::UserRegister.as_str(), post(register))
        .route(RoutesPathes::UserLogin.as_str(), post(login))
        .with_state(pool)
}
