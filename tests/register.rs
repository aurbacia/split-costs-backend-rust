#[cfg(test)]
#[macro_use]
mod common;
mod tests {
    use argon2::{
        Argon2,
        password_hash::{PasswordHash, PasswordVerifier},
    };
    use axum::http::StatusCode;
    use diesel::dsl::exists;
    use diesel::{RunQueryDsl, prelude::*, select};
    use paste::paste;
    use split_costs_rust_backend::database::establish_connection;
    use split_costs_rust_backend::routes::register::Payload;
    use split_costs_rust_backend::schema::users::dsl::*;
    use split_costs_rust_backend::server::RoutesPathes;

    use crate::common;

    fn random_payload() -> Payload {
        Payload {
            email: fakeit::contact::email(),
            display_name: fakeit::name::full(),
            password: fakeit::password::generate(true, true, true, 32),
        }
    }

    #[tokio::test]
    async fn can_register_user() {
        let server = common::setup();
        let payload = random_payload();
        server
            .post(RoutesPathes::UserRegister.as_str())
            .json(&payload)
            .await
            .assert_status(StatusCode::CREATED);

        let pool = establish_connection();
        let conn = pool.get().await.unwrap();
        let exists = conn
            .interact(move |conn| {
                select(exists(users.filter(email.eq(&payload.email))))
                    .get_result::<bool>(conn)
                    .unwrap()
            })
            .await
            .unwrap();

        assert_eq!(exists, true)
    }

    #[tokio::test]
    async fn cant_use_this_same_email_two_times() {
        let server = common::setup();
        let payload = random_payload();
        server
            .post(RoutesPathes::UserRegister.as_str())
            .json(&payload)
            .await
            .assert_status(StatusCode::CREATED);

        server
            .post(RoutesPathes::UserRegister.as_str())
            .json(&Payload {
                email: payload.email,
                ..random_payload()
            })
            .await
            .assert_status_unprocessable_entity();
    }

    #[tokio::test]
    async fn password_can_be_verified() {
        let server = common::setup();
        let payload = random_payload();
        server
            .post(RoutesPathes::UserRegister.as_str())
            .json(&payload)
            .await
            .assert_status(StatusCode::CREATED);

        let pool = establish_connection();
        let conn = pool.get().await.unwrap();
        let password_hash = conn
            .interact(move |conn| {
                users
                    .select(password)
                    .filter(email.eq(&payload.email))
                    .get_result::<String>(conn)
                    .unwrap()
            })
            .await
            .unwrap();

        assert!(
            Argon2::default()
                .verify_password(
                    payload.password.as_bytes(),
                    &PasswordHash::new(&password_hash).unwrap()
                )
                .is_ok()
        );
    }

    test_required_json_field!(email, password, display_name; RoutesPathes::UserRegister.as_str());
    test_valid_json_field!(email RoutesPathes::UserRegister.as_str(); fakeit::words::sentence(3).replace(" ", "-").to_lowercase());
    test_valid_json_field!(password RoutesPathes::UserRegister.as_str(); String::from("ABCD"));
    test_valid_json_field!(display_name RoutesPathes::UserRegister.as_str(); String::from("AB"));
}
