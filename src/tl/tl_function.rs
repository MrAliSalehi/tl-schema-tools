use serde::{Deserialize, Serialize};
use crate::tl::tl_parameter::TlParameter;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TlFunction {
    pub id: String,
    pub name: String,
    pub parameters: Vec<TlParameter>,
    pub return_type: String,
    pub inner_return_type:Option<String>
}