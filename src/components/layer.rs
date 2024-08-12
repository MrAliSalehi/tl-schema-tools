use axum::{extract::{Path, Json, State}, response::IntoResponse, Router, routing::{get, post}};
use axum_valid::{Validated};
use serde_json::json;
use crate::{app_state::AppState, components::{ApiResponse, root}, db, models::{requests::SearchLayerRequest}};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/ids", get(layer_ids))
        .route("/dates", get(layer_release_dates))
        .route("/namespaces", get(get_namespaces))
        .route("/types", get(get_types))
        .route("/:id", get(get_layer))
        .route("/:id/compact", get(get_compact_layer))
        .route("/:id/namespace", get(get_namespace_in_layer))
        .route("/:id/type", get(types_in_layer))
        .route("/search", post(search_in_layer))
        .route("/search/filters", get(get_search_filters))
        .route("/search/ready", get(engine_ready))
        .with_state(state)
}

async fn get_types(State(state): State<AppState>) -> impl IntoResponse {
    let ns = state.schema_manager.get_type_names(None);
    ApiResponse::ok("", Some(json!(ns)))
}
async fn types_in_layer(Path(layer_id): Path<i32>, State(state): State<AppState>) -> impl IntoResponse {
    let ns = state.schema_manager.get_type_names(Some(layer_id));
    ApiResponse::ok("", Some(json!(ns)))
}
async fn get_namespaces(State(state): State<AppState>) -> impl IntoResponse {
    let ns = state.schema_manager.get_namespace(None);
    ApiResponse::ok("", Some(json!(ns)))
}
async fn get_namespace_in_layer(Path(layer_id): Path<u32>, State(state): State<AppState>) -> impl IntoResponse {
    let ns = state.schema_manager.get_namespace(Some(layer_id as _));
    ApiResponse::ok("", Some(json!(ns)))
}
async fn get_search_filters(State(state): State<AppState>) -> impl IntoResponse {
    state.schema_manager.search_filters().await
        .map(|f| ApiResponse::ok(format!("found {} filters.", f.len()), Some(json!({"filters":f}))))
        .map_err(|e| ApiResponse::internal(e.to_string()))
}
async fn engine_ready(State(state): State<AppState>) -> impl IntoResponse {
    state.schema_manager.engine_ready().await
        .map(|e| ApiResponse::ok("", Some(json!({"is_ready":e}))))
        .map_err(|e| ApiResponse::internal(e.to_string()))
}
async fn search_in_layer(State(state): State<AppState>, req: Validated<Json<SearchLayerRequest>>) -> impl IntoResponse {
    state.schema_manager.search(&req.into_inner()).await
        .map(|res| ApiResponse::ok(res.to_string(), Some(json!({"search_results":res.results}))))
        .map_err(|e| ApiResponse::internal(e.to_string()))
}
async fn layer_release_dates(State(state): State<AppState>) -> impl IntoResponse {
    let mut dates = state.schema_manager.release_dates();
    dates.sort_by(|d, d2| d2.release_date.cmp(&d.release_date));
    ApiResponse::ok("", Some(json!({"release_dates":dates})))
}
async fn get_compact_layer(Path(layer_id): Path<u32>, State(state): State<AppState>) -> impl IntoResponse {
    let layer = state.schema_manager.get_compact_layer(layer_id as _);
    if layer.is_empty() {
        ApiResponse::not_found(format!("layer {layer_id} doesn't exist or it's not loaded yet"))
    } else {
        ApiResponse::ok("", Some(json!({"compact_layer": layer})))
    }
}
async fn get_layer(Path(layer_id): Path<u32>, State(state): State<AppState>) -> impl IntoResponse {
    state.schema_manager.get_layer(layer_id as _)
        .map(|l| ApiResponse::ok("", Some(json!({"layer":l}))))
        .unwrap_or(ApiResponse::not_found(format!("layer {layer_id} doesn't exist or it's not loaded yet")))
}
async fn layer_ids(State(state): State<AppState>) -> impl IntoResponse {
    db::tl_layer::get_ids(&state.db).await
        .map(|mut ids| {
            ids.sort();
            ApiResponse::ok(format!("there are a total {} layers", ids.len()), Some(json!({"layers":ids})))
        })
        .map_err(|e| ApiResponse::internal(e.to_string()))
}
