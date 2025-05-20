use super::struct_def::WebStateSharedBag;

use axum::{http::StatusCode, response::IntoResponse, routing::get};

pub fn user_router() -> axum::Router<WebStateSharedBag> {
    axum::Router::new().route("/xprofile", get(user_profile))
}

async fn user_profile() -> impl IntoResponse {
    (StatusCode::OK, "TEST: Here is user profile").into_response()
}
