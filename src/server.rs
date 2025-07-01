use axum::{Router, routing::get, routing::post};

use crate::{
    database::establish_connection,
    routes::{groups::groups_get, groups::groups_post, login::login, register::register},
};

pub enum RoutesPathes {
    UserRegister,
    UserLogin,
    Groups,
}

impl RoutesPathes {
    pub fn as_str(&self) -> &'static str {
        match self {
            RoutesPathes::UserRegister => "/user/register",
            RoutesPathes::UserLogin => "/user/login",
            RoutesPathes::Groups => "/groups",
        }
    }
}

pub fn create_server() -> Router {
    let pool = establish_connection();

    Router::new()
        .route(RoutesPathes::UserRegister.as_str(), post(register))
        .route(RoutesPathes::UserLogin.as_str(), post(login))
        .route(RoutesPathes::Groups.as_str(), post(groups_post))
        .route(RoutesPathes::Groups.as_str(), get(groups_get))
        .with_state(pool)
}
