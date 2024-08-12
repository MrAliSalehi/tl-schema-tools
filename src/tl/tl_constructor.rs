use serde::{Deserialize, Serialize};
use crate::tl::tl_parameter::TlParameter;

/// a tl object
#[derive(Debug, Serialize,Deserialize)]
pub struct TlConstructor {
    pub id: String,
    pub name: String,
    pub namespace:Option<String>,
    pub parameters: Vec<TlParameter>,
}