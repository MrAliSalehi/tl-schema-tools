use axum::extract::State;
use axum::{Json, Router};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum_valid::Validated;
use serde_json::json;
use crate::app_state::AppState;
use crate::components::{ApiResponse, root};
use crate::models::compact_schema::DefinitionType;
use crate::models::requests::{GetByNameRequest, GetNamespaceRequest,  HistoryRequest};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(root).post(get_by_name))
        .route("/namespace", post(get_namespace))
        .route("/history", post(history))
        .with_state(state)
}
async fn history(State(state): State<AppState>, req: Validated<Json<HistoryRequest>>) -> impl IntoResponse {
    let h = state.schema_manager.history(&req.name, DefinitionType::Object);
    ApiResponse::ok("", Some(json!(h)))
}

async fn get_namespace(State(state): State<AppState>, req: Validated<Json<GetNamespaceRequest>>) -> impl IntoResponse {
    state.schema_manager.get_namespace_objects(&req.into_inner())
        .map(|f| ApiResponse::ok("", Some(json!(f))))
        .unwrap_or_else(|| ApiResponse::not_found("could not find the layer id or namespace"))
}

async fn get_by_name(State(state): State<AppState>, req: Validated<Json<GetByNameRequest>>) -> impl IntoResponse {
    state.schema_manager.get_object(&req.into_inner()).await
        .map(|r| ApiResponse::ok(format!("total objects {}", r.count()), Some(json!({"result":r}))))
        .map_err(|e| ApiResponse::internal(e.to_string()))
}
