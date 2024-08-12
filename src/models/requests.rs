use serde::Deserialize;
use validify::Validify;

#[derive(Deserialize, Validify, Default)]
#[serde(default)]
pub struct SearchLayerRequest {
    #[validate(length(min = 3, max = 100, message = "length must be between 3 and 100"))]
    #[modify(trim)]
    pub query: String,
    #[validate(range(min = 1.0, max = 1000.0, message = "id must be between 1 and 1000"))]
    pub layer_id: Option<u32>,
    #[modify(trim, lowercase)]
    #[serde(default)]
    pub filter: Vec<String>,
    #[validate(range(min = 1.0, max = 300.0, message = "limit must be between 1 and 300"))]
    pub limit: Option<usize>,
    #[serde(default)]
    pub highlight: bool,
    #[validate(length(min = 1, max = 15, message = "length must be between 1 and 15"))]
    #[modify(trim)]
    #[serde(default)]
    pub highlight_prefix: Option<String>,
    #[validate(length(min = 1, max = 15, message = "length must be between 1 and 15"))]
    #[modify(trim)]
    #[serde(default)]
    pub highlight_postfix: Option<String>,
}
#[derive(Deserialize, Validify, Default)]
#[serde(default)]
pub struct GetByNameRequest {
    #[validate(length(min = 3, max = 100, message = "length must be between 3 and 100"))]
    #[modify(trim)]
    pub name: String,
    #[validate(range(min = 1.0, max = 1000.0, message = "id must be between 1 and 1000"))]
    pub layer_id: Option<u32>,
    #[serde(default)]
    pub mode: FetchMode,
    #[validate(range(min = 1.0, max = 300.0, message = "limit must be between 1 and 300"))]
    pub limit: Option<usize>,
}

#[derive(Deserialize, Validify, Default)]
#[serde(default)]
pub struct HistoryRequest {
    #[validate(length(min = 3, max = 100, message = "length must be between 3 and 100"))]
    #[modify(trim)]
    pub name: String,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FetchMode {
    #[default]
    Compact,
    Full,
}
#[derive(Deserialize, Validify)]
pub struct GetNamespaceRequest {
    #[validate(range(min = 1.0, max = 1000.0, message = "id must be between 1 and 1000"))]
    pub layer_id: u32,
    #[validate(length(min = 3, max = 100, message = "length must be between 3 and 100"))]
    #[modify(trim)]
    pub namespace: String,
}

