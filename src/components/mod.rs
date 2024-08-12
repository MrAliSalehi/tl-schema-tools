use axum::{http::StatusCode, response::{IntoResponse, Response}, Json, Router, routing::get};
use serde_json::{json, Value};
use crate::app_state::AppState;

mod layer;
mod function;
mod object;
mod ty;
pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .nest("/layer", layer::routes(state.clone()))
        .nest("/function", function::routes(state.clone()))
        .nest("/object", object::routes(state.clone()))
        .nest("/type", ty::routes(state.clone()))
}

pub async fn root() -> impl IntoResponse {
    axum::response::Html::from(r#"
    <html>
    <p>hello</p>
    </html>
    "#)
}

pub(super) struct ApiResponse {
    pub message: String,
    pub data: Option<Value>,
    pub status: StatusCode,
}

impl ApiResponse {
    pub(crate) fn internal<M: Into<String>>(msg: M) -> Self {
        Self {
            message: msg.into(),
            data: None,
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    pub(crate) fn not_found<M: Into<String>>(msg: M) -> Self {
        Self {
            message: msg.into(),
            data: None,
            status: StatusCode::NOT_FOUND,
        }
    }
    pub(crate) fn ok<M: Into<String>>(message: M, data: Option<Value>) -> Self {
        Self {
            message: message.into(),
            data,
            status: StatusCode::OK,
        }
    }
    pub fn bad_request<M: Into<String>>(message: M) -> Self {
        Self {
            data: None,
            message: message.into(),
            status: StatusCode::BAD_REQUEST,
        }
    }
}
impl IntoResponse for ApiResponse {
    fn into_response(self) -> Response {
        (self.status, Json(json!({
            "message":self.message,
            "data":self.data,
        }))).into_response()
    }
}