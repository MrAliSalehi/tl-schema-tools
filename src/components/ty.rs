use axum::extract::State;
use axum::{Json, Router};
use axum::response::IntoResponse;
use axum::routing::{get};
use axum_valid::Validated;
use serde_json::json;
use crate::app_state::AppState;
use crate::components::{ApiResponse, root};
use crate::models::requests::{GetByNameRequest};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(root).post(get_by_name))
        .with_state(state)
}
async fn get_by_name(State(state): State<AppState>, req: Validated<Json<GetByNameRequest>>) -> impl IntoResponse {
    let t = state.schema_manager.get_types(&req.into_inner());
    ApiResponse::ok("", Some(json!(t)))
}