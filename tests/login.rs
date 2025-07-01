#[macro_use]
pub mod common;
#[macro_use]
mod tests {
    use axum::http::header::SET_COOKIE;
    use paste::paste;
    use split_costs_rust_backend::extractors::session::decode_jwt_user_session;
    use split_costs_rust_backend::routes::login::Payload;
    use split_costs_rust_backend::server::RoutesPathes;

    use crate::common::common;
    fn random_payload() -> Payload {
        Payload {
            email: fakeit::contact::email(),
            password: fakeit::password::generate(true, true, true, 32),
        }
    }

    #[tokio::test]
    async fn can_login_user_and_returns_valid_token_as_cookie() {
        let server = common::setup();
        let payload_register = common::random_payload_register();
        server
            .post(RoutesPathes::UserRegister.as_str())
            .json(&payload_register)
            .await;

        let response_login = server
            .post(RoutesPathes::UserLogin.as_str())
            .json(&Payload {
                email: payload_register.email.clone(),
                password: payload_register.password,
            })
            .await;

        response_login.assert_status_ok();
        response_login.assert_contains_header(SET_COOKIE);

        let session_token = response_login // NOTE: CookieJar doesn't support from_headers on Set-Cookie header
            .header(SET_COOKIE)
            .to_str()
            .unwrap()
            .replace("session_token=", "");

        let claims = decode_jwt_user_session(&session_token);
        assert_eq!(claims.claims.display_name, payload_register.display_name);
        assert_eq!(claims.claims.email, payload_register.email);
    }

    #[tokio::test]
    async fn cant_login_when_user_doesnt_exists() {
        let server = common::setup();
        server
            .post(RoutesPathes::UserLogin.as_str())
            .json(&random_payload())
            .await
            .assert_status_unauthorized()
    }

    #[tokio::test]
    async fn cant_login_when_password_is_wrong() {
        let server = common::setup();
        let payload_register = common::random_payload_register();
        server
            .post(RoutesPathes::UserRegister.as_str())
            .json(&payload_register)
            .await;

        server
            .post(RoutesPathes::UserLogin.as_str())
            .json(&Payload {
                email: payload_register.email.clone(),
                password: String::from("wrong password"),
            })
            .await
            .assert_status_unauthorized()
    }

    test_required_json_field!(email, password; RoutesPathes::UserLogin.as_str());
    test_valid_json_field!(email RoutesPathes::UserLogin.as_str(); fakeit::words::sentence(3).replace(" ", "-").to_lowercase());
    test_valid_json_field!(password RoutesPathes::UserLogin.as_str(); fakeit::words::word()[..3].to_string());
}
