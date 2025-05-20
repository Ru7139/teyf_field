use super::minor_router;
use super::struct_def::{WebStateResponse, WebStateSharedBag};

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, extract::State, routing::get};

pub fn basic_server(shared_bag: WebStateSharedBag) -> axum::Router {
    axum::Router::new()
        .route("/xindex", get(|| async { "TEST: Hello, world" }))
        .route("/display_full_state_bag", get(display_full_bag))
        .route(
            "/change_destination_random",
            get(change_destination_25_random),
        )
        .nest("/user", minor_router::user_router())
        .with_state(shared_bag)
}

async fn display_full_bag(State(bag): State<WebStateSharedBag>) -> Json<WebStateResponse> {
    Json(bag.into_web_state_response().await.unwrap())
}

async fn change_destination_25_random(State(bag): State<WebStateSharedBag>) -> impl IntoResponse {
    bag.change_to_random_location(25f64).await;
    let xyz = bag.get_destination_xyz().await;
    (
        StatusCode::ACCEPTED,
        format!("New destination location: {},{},{}", xyz.0, xyz.1, xyz.2),
    )
        .into_response()
}
