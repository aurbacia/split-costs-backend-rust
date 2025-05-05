use axum_test::TestServer;
use split_costs_rust_backend::server::create_server;

pub fn setup() -> TestServer {
    dotenvy::dotenv().ok();
    let app = create_server();
    TestServer::new(app).unwrap()
}

macro_rules! test_required_json_field {
    ($($field_name:ident),*; $route:expr) => {
        paste! {
            $(
                #[tokio::test]
                async fn [<field_ $field_name _is_required>]() {
                    let server = $crate::common::setup();
                    let mut payload = random_payload();
                    payload.$field_name = Default::default();
                    server
                        .post($route)
                        .json(&payload)
                        .await
                        .assert_status_bad_request();
                }

                #[tokio::test]
                async fn [<field_ $field_name _cant_be_empty>]() {
                    let server = $crate::common::setup();
                    let mut payload = random_payload();
                    payload.$field_name = String::from("");
                    server
                        .post($route)
                        .json(&payload)
                        .await
                        .assert_status_bad_request();
                }
            )*

        }
    };
}

macro_rules! test_valid_json_field {
    ($field_name:ident $route:expr; $incorrect_data:expr) => {
        paste! {
            #[tokio::test]
            async fn [<field_ $field_name _has_to_be_valid>]() {
                let server = $crate::common::setup();
                let mut payload = random_payload();
                payload.$field_name = $incorrect_data;
                server
                    .post($route)
                    .json(&payload)
                    .await
                    .assert_status_bad_request();
            }
        }
    };
}
