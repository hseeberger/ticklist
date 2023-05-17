use axum::{response::IntoResponse, routing::get, Router};
use shuttle_axum::ShuttleAxum;

#[shuttle_runtime::main]
async fn axum() -> ShuttleAxum {
    let router = Router::new().route("/", get(hello_world));

    Ok(router.into())
}

async fn hello_world() -> impl IntoResponse {
    "Hello, world!"
}
