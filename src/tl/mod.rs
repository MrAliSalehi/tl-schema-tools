use std::collections::HashMap;
use std::str::Lines;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use crate::continue_if;
use crate::models::tl_layer::TlLayer;
use crate::tl::tl_constructor::TlConstructor;
use crate::tl::tl_function::TlFunction;
use crate::tl::tl_parameter::{parse_parameter};
use crate::tl::tl_type::TlType;

pub mod tl_parameter;
pub mod tl_constructor;
pub mod tl_type;
pub mod tl_function;
pub mod schema_manager;
const IGNORED_DEFINITIONS: [&str; 6] = ["boolFalse", "boolTrue", "true", "error", "vector", "null"];

#[derive(Serialize, Deserialize, Debug)]
pub struct TlSchema {
    pub layer_id: i32,
    pub release_date: NaiveDateTime,
    pub objects: Vec<TlType>,
    pub functions: HashMap<String, Vec<TlFunction>>,
}
pub fn parse_schema(layer: TlLayer) -> TlSchema {
    let definitions = layer.layer
        .lines()
        .filter(|l|
        !l.is_empty() &&
            !IGNORED_DEFINITIONS.iter().any(|ig| l.starts_with(ig)))
        .collect::<Vec<_>>()
        .join("\n");

    //[0] is mtproto definition and [1] is the api
    let sp = definitions.split("///////// Main application API").collect::<Vec<_>>();
    let definitions = (if sp.len() == 2 { sp[1].to_owned() } else { definitions })
        .lines()
        .filter(|l| !l.starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
        .replace("---types---", "");

    let spl = definitions.split("---functions---").collect::<Vec<_>>();
    let objects = spl[0].lines();
    let functions = spl[1].lines();
    let objects = parse_objects(objects);
    let functions = parse_functions(functions);

    TlSchema { layer_id: layer.layer_id, objects, functions, release_date: layer.release_date }
}

fn parse_functions(functions: Lines) -> HashMap<String, Vec<TlFunction>> {
    let mut map = HashMap::new();
    for function in functions {
        continue_if!(function.is_empty());
        let mut spl = function.split("=");
        let definition = spl.next().unwrap();
        let return_type = spl.next().unwrap().trim().replace(";", "");

        let inner_return_type = if return_type.contains('<') {
            Some(return_type.split("<").last().unwrap().split(">").next().unwrap().to_owned())
        } else { None };

        let mut spl = definition.split("#");

        let name = spl.next().unwrap().trim().to_string();
        let name_spl = name.split(".").collect::<Vec<_>>();
        let k = if name_spl.len() == 2 { name_spl[0].to_string() } else { name.to_owned() };
        let (id, parameters) = parse_parameter(&spl.collect::<Vec<_>>().join("#"));
        let f = TlFunction { id, name, parameters, return_type, inner_return_type };
        map.entry(k).or_insert(vec![]).push(f);
    }
    let mut singles = vec![];
    for v in &mut map.values_mut() {
        if v.len() == 1 {
            singles.push(v.pop().unwrap())
        };
    }
    let mut map = map.into_iter()
        .filter(|a| !a.1.is_empty())
        .collect::<HashMap<_, _>>();
    map.entry("Others".to_owned()).or_insert(singles);
    map
}

fn parse_objects(objects: Lines) -> Vec<TlType> {
    let mut map = HashMap::new();

    for object in objects {
        continue_if!(object.is_empty());
        let mut spl = object.split("=");
        let definition = spl.next().unwrap();

        let category = spl.next().unwrap().trim().replace(";", "");

        let mut spl = definition.split("#");
        let name = spl.next().unwrap().trim().to_string();

        let name_spl = name.split(".").collect::<Vec<_>>();
        let namespace = if name_spl.len() == 2 {
            Some(name_spl[0].to_string())
        } else {
            None
        };
        let (id, parameters) = parse_parameter(&spl.collect::<Vec<_>>().join("#"));
        let con = TlConstructor { parameters, id, name, namespace };

        map.entry(category).or_insert(vec![]).push(con);
    }

    map.into_iter()
        .map(|(name, constructors)| TlType { constructors, name })
        .collect()
}

