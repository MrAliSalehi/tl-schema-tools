use std::collections::HashSet;
use std::time::Duration;
use dotenv::var;
use itertools::Itertools;
use meilisearch_sdk::{client::Client, search::{SearchResults, Selectors}, task_info::TaskInfo};
use serde::de::DeserializeOwned;
use crate::{
    models::{
        responses::{CompactTlDefinitionResponse, FunctionHistory, FunctionHistoryResponse, GetFuncResponse, GetFunction, GetObject, GetObjectResponse, HistoryResponse, Namespace, ObjectHistory, ObjectHistoryResponse, ObjectUsage, SearchResponse, TypeResponse},
        layer_release_date::LayerReleaseDate,
        tl_layer::TlLayer,
        requests::{FetchMode, GetByNameRequest, GetNamespaceRequest, SearchLayerRequest},
        compact_schema::{CompactTlConstructor, CompactTlDefinition, DefinitionType},
    },
    tl::{
        self,
        tl_constructor::TlConstructor,
        tl_function::TlFunction,
        tl_parameter::TlParameter,
        TlSchema,
    },
};
use crate::models::compact_schema::RefCompactTlConstructor;
use crate::models::responses::{GetTypeCompact, GetTypeFull, GetTypeResponse};

pub struct SchemaManager {
    schemas: Vec<TlSchema>,
    compact_definitions: Vec<CompactTlDefinition>,
    meilisearch: Client,
    init_task_info: TaskInfo,
}


impl SchemaManager {
    pub async fn new(layers: Vec<TlLayer>, meilisearch: Client) -> eyre::Result<Self> {
        let mut schemas = layers
            .into_iter()
            .map(tl::parse_schema)
            .collect::<Vec<_>>();

        schemas.sort_by(|s, s2| s.layer_id.cmp(&s2.layer_id));

        let compact_definitions = Self::create_compact_definitions(&schemas);
        let index = meilisearch.index("schema");

        let task_info = if var("REPLACE_DATA")?.parse::<bool>().unwrap_or(false) {
            let info = meilisearch.delete_index("schema").await?;
            info.wait_for_completion(&meilisearch, Some(Duration::from_secs(2)), Some(Duration::from_secs(10))).await?;
            index.add_documents(&compact_definitions, Some("id")).await?
        } else {
            log::trace!("skipped index creation");
            crate::prelude::DEFAULT_TASK_INFO
        };
        println!("finished initializing");
        index.set_filterable_attributes(["layer_id", "definition_id", "name", "definition_type", "return_type", "namespace"]).await?;
        Ok(Self { schemas, compact_definitions, meilisearch, init_task_info: task_info })
    }

    pub fn get_types(&self, req: &GetByNameRequest) -> GetTypeResponse {
        let limit = req.limit.unwrap_or(30);
        let layers_to_iter = self.filter_schema_by_id(req.layer_id.map(|a| a as i32));

        match req.mode {
            FetchMode::Compact => {
                let mut _types = vec![];
                for layer in layers_to_iter {
                    let Some(tl_type) = layer.objects.iter().find(|a| a.name.eq(&req.name)) else {
                        continue;
                    };
                    let objects = tl_type.constructors
                        .iter()
                        .take(limit)
                        .map(RefCompactTlConstructor::from)
                        .collect();
                    _types.push(GetTypeCompact { objects, layer_id: layer.layer_id });
                }
                GetTypeResponse::CompactMode(_types)
            }
            FetchMode::Full => {
                let mut _types = vec![];
                for layer in layers_to_iter {
                    let Some(tl_type) = layer.objects.iter().find(|a| a.name.eq(&req.name)) else {
                        continue;
                    };
                    if _types.len() >= limit { break; }
                    _types.push(GetTypeFull { layer_id: layer.layer_id, objects: &tl_type.constructors });
                }
                GetTypeResponse::FullMode(_types)
            }
        }
    }
    pub fn get_type_names(&self, layer_id: Option<i32>) -> Vec<TypeResponse> {
        self.filter_schema_by_id(layer_id)
            .iter()
            .map(|f| TypeResponse { layer_id: f.layer_id, types: f.objects.iter().map(|a| &a.name).collect() })
            .collect()
    }

    pub fn get_namespace(&self, layer_id: Option<i32>) -> Vec<Namespace> {
        let layers_to_iter = self.filter_schema_by_id(layer_id);

        let mut res = vec![];
        for schema in layers_to_iter {
            let mut map = HashSet::<&String>::new();
            for tl_type in &schema.objects {
                for ctor in &tl_type.constructors {
                    if let Some(ns) = &ctor.namespace {
                        map.insert(ns);
                    }
                }
            }
            res.push(Namespace {
                layer_id: schema.layer_id as _,
                function_ns: schema.functions.keys().collect(),
                object_ns: map.into_iter().collect::<Vec<_>>(),
            });
        }
        res
    }

    pub fn get_namespace_functions(&self, req: &GetNamespaceRequest) -> Option<&Vec<TlFunction>> {
        let li = req.layer_id as i32;
        let schema = self.schemas.iter().find(|f| f.layer_id == li)?;
        schema.functions.get(&req.namespace)
    }

    pub fn get_namespace_objects(&self, req: &GetNamespaceRequest) -> Option<Vec<&TlConstructor>> {
        let li = req.layer_id as i32;
        let schema = self.schemas.iter().find(|f| f.layer_id == li)?;
        let mut res = vec![];
        for tl_type in &schema.objects {
            for ctor in &tl_type.constructors {
                if ctor.namespace.as_ref().is_some_and(|f| f.eq(&req.namespace)) {
                    res.push(ctor);
                }
            }
        }
        Some(res)
    }

    pub fn history(&self, name: &str, definition_type: DefinitionType) -> HistoryResponse {
        match definition_type {
            DefinitionType::Function => self.get_function_history(name).map(HistoryResponse::Function),
            DefinitionType::Object => self.get_object_history(name).map(HistoryResponse::Object)
        }.unwrap_or(HistoryResponse::Empty)
    }

    pub async fn get_object(&self, req: &GetByNameRequest) -> eyre::Result<GetObjectResponse> {
        let limit = req.limit.unwrap_or(30);
        Ok(match req.mode {
            FetchMode::Compact => {
                let result = self.get_definitions::<CompactTlConstructor>(&req.name, req.layer_id, DefinitionType::Object, limit).await?;
                GetObjectResponse::CompactMode(result.hits.into_iter().map(|a| a.result).collect())
            }
            FetchMode::Full => GetObjectResponse::FullMode(self.get_obj_full(Some(limit), &req.name, req.layer_id))
        })
    }

    pub async fn get_func(&self, req: &GetByNameRequest) -> eyre::Result<GetFuncResponse> {
        let limit = req.limit.unwrap_or(30);
        Ok(match req.mode {
            FetchMode::Compact => {
                let result = self.get_definitions::<CompactTlDefinition>(&req.name, req.layer_id, DefinitionType::Function, limit).await?;
                GetFuncResponse::CompactMode(result.hits.into_iter().map(|a| a.result).collect())
            }
            FetchMode::Full => GetFuncResponse::FullMode(self.get_func_full(Some(limit), &req.name, req.layer_id))
        })
    }

    pub async fn search(&self, req: &SearchLayerRequest) -> eyre::Result<SearchResponse> {
        let filter = if let Some(layer_id) = req.layer_id {
            format!("layer_id = {layer_id}")
        } else { String::default() };
        let search_on = if req.filter.is_empty() { vec!["*"] } else { req.filter.iter().map(|s| s.as_str()).collect::<Vec<_>>() };
        let prefix = if let Some(prefix) = &req.highlight_prefix { prefix.as_str() } else { "" };
        let postfix = if let Some(postfix) = &req.highlight_postfix { postfix.as_str() } else { "" };

        self.meilisearch
            .index("schema")
            .search()
            .with_query(&req.query)
            .with_attributes_to_search_on(&search_on)
            .with_filter(&filter)
            .with_limit(req.limit.unwrap_or(10))
            .with_show_ranking_score(true)
            .with_highlight_pre_tag(prefix)
            .with_highlight_post_tag(postfix)
            .with_attributes_to_highlight(Selectors::All)
            .execute::<CompactTlDefinition>().await
            .map(|r|
            SearchResponse {
                results: r.hits
                    .into_iter()
                    .map(|a| CompactTlDefinitionResponse::from(a, req.highlight))
                    .collect::<Vec<_>>(),
                process_time: r.processing_time_ms,
                query: r.query,
                total_hits: r.estimated_total_hits.unwrap_or(0),
            })
            .map_err(|e| eyre::Report::msg(e.to_string()))
    }

    pub async fn search_filters(&self) -> eyre::Result<Vec<String>> {
        Ok(self.meilisearch.index("schema").get_filterable_attributes().await?)
    }

    pub async fn engine_ready(&self) -> eyre::Result<bool> {
        Ok(self.meilisearch.get_task(&self.init_task_info).await?.is_success())
    }

    pub fn get_layer(&self, layer_id: i32) -> Option<&TlSchema> {
        self.schemas.iter().find(|s| s.layer_id == layer_id)
    }

    pub fn get_compact_layer(&self, layer_id: i32) -> Vec<&CompactTlDefinition> {
        self.compact_definitions.iter().filter(|s| s.layer_id == layer_id).collect::<Vec<_>>()
    }

    pub fn release_dates(&self) -> Vec<LayerReleaseDate> {
        self.schemas
            .iter()
            .map(|s| LayerReleaseDate { release_date: s.release_date.date(), layer_id: s.layer_id })
            .collect()
    }

    async fn get_definitions<T: 'static + DeserializeOwned + Send + Sync>(&self, name: &str, layer_id: Option<u32>, d: DefinitionType, limit: usize) -> eyre::Result<SearchResults<T>> {
        let mut filter = vec![format!("name={name}"), format!("definition_type={d}")];
        if let Some(id) = layer_id {
            filter.push(format!("layer_id={id}"));
        }
        let filter = filter.iter().map(|s| s.as_str()).collect::<Vec<_>>();
        Ok(self.meilisearch
            .index("schema")
            .search()
            .with_array_filter(filter)
            .with_limit(limit)
            .execute::<T>().await?)
    }

    fn get_func_full(&self, limit: Option<usize>, name: &str, layer_id: Option<u32>) -> Vec<GetFunction> {
        let layers_to_iter = self.filter_schema_by_id(layer_id.map(|f| f as i32));


        let r = layers_to_iter.iter()
            .flat_map(|layer| layer.functions.values()
                .flat_map(|functions| functions
                    .iter()
                    .filter(|f| f.name.eq(name))
                    .map(|f| GetFunction { function: f, layer_id: layer.layer_id as _ })
                    .collect::<Vec<_>>())
                .collect::<Vec<_>>());

        if let Some(limit) = limit {
            r.take(limit).collect()
        } else {
            r.collect()
        }
    }

    fn get_obj_full(&self, limit: Option<usize>, name: &str, layer_id: Option<u32>) -> Vec<GetObject> {
        let layers_to_iter = self.filter_schema_by_id(layer_id.map(|f| f as i32));

        let r = layers_to_iter.iter()
            .flat_map(|layer| layer.objects.iter()
                .flat_map(|tl_type| tl_type.constructors
                    .iter()
                    .filter(|f| f.name.eq(name))
                    .map(|ctor| (&tl_type.name, GetObject { obj: ctor, layer_id: layer.layer_id as _, usages: vec![] }))
                    .collect::<Vec<_>>()
                )
                .collect::<Vec<_>>());

        let mut objects = if let Some(limit) = limit {
            r.take(limit).collect::<Vec<_>>()
        } else {
            r.collect()
        };


        for (ns, obj) in objects.iter_mut() {
            let Some(related_layer) = layers_to_iter.iter().find(|l| l.layer_id == obj.layer_id as i32) else {
                continue;
            };
            for funcs in related_layer.functions.values() {
                for func in funcs {
                    if let Some(inner) = &func.inner_return_type {
                        if inner.eq(*ns) {
                            obj.usages.push(ObjectUsage::ViaNamespace { tl_function: func, is_inner: true });
                        } else if inner.eq(&obj.obj.name) {
                            obj.usages.push(ObjectUsage::ReturnType { tl_function: func, is_inner: true });
                        }
                    }
                    if func.return_type.eq(*ns) {
                        obj.usages.push(ObjectUsage::ViaNamespace { tl_function: func, is_inner: false });
                    } else if func.return_type.eq(&obj.obj.name) {
                        obj.usages.push(ObjectUsage::ReturnType { tl_function: func, is_inner: false });
                    }


                    for param in &func.parameters {
                        if let Some(inner) = &param.inner_type {
                            if inner.eq(*ns) {
                                obj.usages.push(ObjectUsage::ViaNamespace { tl_function: func, is_inner: true });
                            } else if inner.eq(&obj.obj.name) {
                                obj.usages.push(ObjectUsage::Param { tl_function: func, is_inner: true });
                            }
                        }
                        if param._type.eq(*ns) {
                            obj.usages.push(ObjectUsage::ViaNamespace { tl_function: func, is_inner: false });
                        } else if param._type.eq(&obj.obj.name) {
                            obj.usages.push(ObjectUsage::Param { tl_function: func, is_inner: false });
                        }
                    }
                }
            }
        }

        objects.into_iter().map(|(_, b)| b).collect()
    }

    fn get_object_history(&self, name: &str) -> Option<ObjectHistoryResponse> {
        let mut objects = self.get_obj_full(None, name, None);
        if objects.is_empty() {
            return None;
        }
        objects.sort_by(|f, f2| f2.layer_id.cmp(&f.layer_id));

        let mut history = vec![ObjectHistory::AddedIn { layer_id: objects.last().unwrap().layer_id }];
        let iter = objects.iter().tuple_windows();
        for (a, b) in iter {
            for param_b in &b.obj.parameters {
                if !a.obj.parameters.iter().any(|f| f.name == param_b.name) {
                    history.push(ObjectHistory::ParamDeleted { layer_id: a.layer_id, name: &param_b.name });
                }
            }
            for param_a in &a.obj.parameters {
                if let Some(found_param) = b.obj.parameters.iter().find(|param_b| param_b.name == param_a.name) {
                    if let Some(diff) = TlParameter::diff(param_a, found_param) {
                        history.push(ObjectHistory::ParamChanged { layer_id: a.layer_id, diff, name: &param_a.name });
                    }
                } else {
                    history.push(ObjectHistory::ParamAdded { layer_id: a.layer_id, param_type: &param_a._type, name: &param_a.name });
                }
            }
        }
        let latest_layer = self.schemas.last().unwrap().layer_id;
        let last_appeared_in = objects.first().cloned().unwrap();
        if last_appeared_in.layer_id as i32 != latest_layer {
            history.push(ObjectHistory::DeletedIn { layer_id: last_appeared_in.layer_id });
        }
        Some(ObjectHistoryResponse { history, last_definition: last_appeared_in })
    }

    fn get_function_history(&self, name: &str) -> Option<FunctionHistoryResponse> {
        let mut functions = self.get_func_full(None, name, None);
        if functions.is_empty() {
            return None;
        }
        functions.sort_by(|f, f2| f2.layer_id.cmp(&f.layer_id));

        let mut history = vec![FunctionHistory::AddedIn { layer_id: functions.last().unwrap().layer_id }];

        let iter = functions.iter().tuple_windows();
        for (a, b) in iter {
            if a.function.return_type != b.function.return_type {
                history.push(FunctionHistory::ReturnTypeChanged { layer_id: a.layer_id, before: &b.function.return_type, after: &a.function.return_type });
            }
            for param_b in &b.function.parameters {
                if !a.function.parameters.iter().any(|f| f.name == param_b.name) {
                    history.push(FunctionHistory::ParamDeleted { layer_id: a.layer_id, name: &param_b.name });
                }
            }
            for param_a in &a.function.parameters {
                if let Some(found_param) = b.function.parameters.iter().find(|param_b| param_b.name == param_a.name) {
                    if let Some(diff) = TlParameter::diff(param_a, found_param) {
                        history.push(FunctionHistory::ParamChanged { layer_id: a.layer_id, diff, name: &param_a.name });
                    }
                } else {
                    history.push(FunctionHistory::ParamAdded { layer_id: a.layer_id, param_type: &param_a._type, name: &param_a.name });
                }
            }
        }
        let latest_layer = self.schemas.last().unwrap().layer_id;
        let last_appeared_in = functions.first().cloned().unwrap();
        if last_appeared_in.layer_id as i32 != latest_layer {
            history.push(FunctionHistory::DeletedIn { layer_id: last_appeared_in.layer_id });
        }
        Some(FunctionHistoryResponse { history, last_definition: last_appeared_in })
    }

    fn filter_schema_by_id(&self, layer_id: Option<i32>) -> Vec<&TlSchema> {
        if let Some(layer_id) = layer_id {
            self.schemas.iter().find(|s| s.layer_id == layer_id)
                .map(|s| vec![s])
                .unwrap_or(vec![])
        } else {
            self.schemas.iter().collect()
        }
    }

    fn create_compact_definitions(schemas: &Vec<TlSchema>) -> Vec<CompactTlDefinition> {
        let mut definitions = vec![];
        for schema in schemas {
            schema.functions.iter().for_each(|(ns, funcs)| {
                funcs.iter().for_each(|f| definitions.push(
                    CompactTlDefinition {
                        id: uuid::Uuid::new_v4(),
                        layer_id: schema.layer_id,
                        name: f.name.to_owned(),
                        return_type: Some(f.return_type.to_owned()),
                        definition_id: f.id.to_owned(),
                        namespace: ns.to_owned(),
                        definition_type: DefinitionType::Function,
                    }
                ));
            });
            schema.objects.iter().for_each(|tl_types| {
                tl_types.constructors.iter().for_each(|f| definitions.push(
                    CompactTlDefinition {
                        id: uuid::Uuid::new_v4(),
                        layer_id: schema.layer_id,
                        name: f.name.to_owned(),
                        return_type: None,
                        definition_id: f.id.to_owned(),
                        namespace: tl_types.name.to_owned(),
                        definition_type: DefinitionType::Object,
                    }
                ));
            });
        }
        definitions
    }
}