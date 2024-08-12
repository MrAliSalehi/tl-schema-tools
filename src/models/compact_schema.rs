use std::fmt::Display;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::tl::tl_constructor::TlConstructor;

#[derive(Serialize, Deserialize, Debug)]
pub struct CompactTlDefinition {
    pub id: Uuid,
    pub layer_id: i32,
    pub definition_id: String,
    pub name: String,
    pub namespace: String,
    pub return_type: Option<String>,
    pub definition_type: DefinitionType,
}
#[derive(Serialize, Deserialize)]
pub struct CompactTlConstructor {
    pub id: String,
    pub name: String,
    pub layer_id: i32,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub enum DefinitionType {
    Function,
    #[default]
    Object,
}

impl Display for DefinitionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            DefinitionType::Function => String::from("Function"),
            DefinitionType::Object => String::from("Object"),
        };
        write!(f, "{}", str)
    }
}

#[derive(Serialize)]
pub struct RefCompactTlConstructor<'a> {
    pub id: &'a String,
    pub name: &'a String,
}

impl<'a> From<&'a TlConstructor> for RefCompactTlConstructor<'a> {
    fn from(value: &'a TlConstructor) -> Self {
        Self { id: &value.id, name: &value.name }
    }
}