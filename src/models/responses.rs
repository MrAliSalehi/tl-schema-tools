use std::fmt::Display;
use meilisearch_sdk::search::{SearchResult};
use serde::{Serialize};
use serde_json::{Map, Value};
use crate::models::compact_schema::{CompactTlConstructor, CompactTlDefinition, DefinitionType, RefCompactTlConstructor};
use crate::tl::tl_constructor::TlConstructor;
use crate::tl::tl_function::TlFunction;

#[derive(Serialize)]
pub struct CompactTlDefinitionResponse {
    pub ranking_score: f64,
    pub layer_id: i32,
    pub definition_id: String,
    pub name: String,
    pub namespace: String,
    pub return_type: Option<String>,
    pub definition_type: DefinitionType,
    pub formated_result: Option<Map<String, Value>>,
}
impl CompactTlDefinitionResponse {
    pub fn from(value: SearchResult<CompactTlDefinition>, highlight: bool) -> Self {
        let formated_result = if highlight { value.formatted_result } else { None };
        Self {
            ranking_score: value.ranking_score.unwrap_or(0.0),
            name: value.result.name,
            return_type: value.result.return_type,
            namespace: value.result.namespace,
            definition_id: value.result.definition_id,
            layer_id: value.result.layer_id,
            definition_type: value.result.definition_type,
            formated_result,
        }
    }
}
pub struct SearchResponse {
    pub results: Vec<CompactTlDefinitionResponse>,
    pub process_time: usize,
    pub total_hits: usize,
    pub query: String,
}
impl Display for SearchResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "processed {} ({} MS). estimated hits: {}.", self.query, self.process_time, self.total_hits)
    }
}
#[derive(Serialize, Debug, Clone)]
pub struct GetFunction<'a> {
    pub function: &'a TlFunction,
    pub layer_id: u32,
}

#[derive(Serialize)]
pub enum GetFuncResponse<'a> {
    CompactMode(Vec<CompactTlDefinition>),
    FullMode(Vec<GetFunction<'a>>),
}
impl<'a> GetFuncResponse<'a> {
    pub fn count(&self) -> usize {
        match self {
            GetFuncResponse::CompactMode(a) => a.len(),
            GetFuncResponse::FullMode(b) => b.len()
        }
    }
}
#[derive(Serialize)]
pub enum GetObjectResponse<'a> {
    CompactMode(Vec<CompactTlConstructor>),
    FullMode(Vec<GetObject<'a>>),
}
impl<'a> GetObjectResponse<'a> {
    pub fn count(&self) -> usize {
        match self {
            GetObjectResponse::CompactMode(a) => a.len(),
            GetObjectResponse::FullMode(b) => b.len()
        }
    }
}
#[derive(Serialize, Debug, Clone)]
pub struct GetObject<'a> {
    pub obj: &'a TlConstructor,
    pub usages: Vec<ObjectUsage<'a>>,
    pub layer_id: u32,
}
#[derive(Serialize, Debug, Clone)]
pub enum ObjectUsage<'a> {
    Param { tl_function: &'a TlFunction, is_inner: bool },
    ReturnType { tl_function: &'a TlFunction, is_inner: bool },
    ViaNamespace { tl_function: &'a TlFunction, is_inner: bool },
}

#[derive(Serialize)]
pub enum HistoryResponse<'a> {
    Function(FunctionHistoryResponse<'a>),
    Object(ObjectHistoryResponse<'a>),
    Empty,
}

#[derive(Serialize)]
pub struct FunctionHistoryResponse<'a> {
    pub history: Vec<FunctionHistory<'a>>,
    pub last_definition: GetFunction<'a>,
}
#[derive(Serialize)]
pub enum FunctionHistory<'a> {
    AddedIn { layer_id: u32 },
    DeletedIn { layer_id: u32 },
    ParamAdded { layer_id: u32, name: &'a str, param_type: &'a str },
    ParamChanged { layer_id: u32, diff: Vec<Diff<'a>>, name: &'a String },
    ParamDeleted { layer_id: u32, name: &'a str },
    ReturnTypeChanged { layer_id: u32, before: &'a str, after: &'a str },
}
#[derive(Serialize)]
pub struct Diff<'a> {
    pub from: String,
    pub to: String,
    pub field_name: &'a str,
}
#[derive(Serialize)]
pub struct ObjectHistoryResponse<'a> {
    pub history: Vec<ObjectHistory<'a>>,
    pub last_definition: GetObject<'a>,
}

#[derive(Serialize)]
pub enum ObjectHistory<'a> {
    AddedIn { layer_id: u32 },
    DeletedIn { layer_id: u32 },
    ParamAdded { layer_id: u32, name: &'a str, param_type: &'a str },
    ParamChanged { layer_id: u32, diff: Vec<Diff<'a>>, name: &'a String },
    ParamDeleted { layer_id: u32, name: &'a str },
}


#[derive(Serialize)]
pub struct Namespace<'a> {
    pub layer_id: u32,
    pub function_ns: Vec<&'a String>,
    pub object_ns: Vec<&'a String>,
}
#[derive(Serialize)]
pub struct TypeResponse<'a> {
    pub layer_id: i32,
    pub types: Vec<&'a String>,
}

#[derive(Serialize)]
pub enum GetTypeResponse<'a> {
    FullMode(Vec<GetTypeFull<'a>>),
    CompactMode(Vec<GetTypeCompact<'a>>),
}

#[derive(Serialize)]
pub struct GetTypeFull<'a> {
    pub layer_id: i32,
    pub objects: &'a Vec<TlConstructor>,
}
#[derive(Serialize)]
pub struct GetTypeCompact<'a> {
    pub layer_id: i32,
    pub objects: Vec<RefCompactTlConstructor<'a>>,
}