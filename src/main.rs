use axum::{Router, routing::post};
use dotenvy::dotenv;

use split_costs_rust_backend::{database::establish_connection, routes::register::register};

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    let pool = establish_connection();
    let app = Router::new()
        .route("/user/register", post(register))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
