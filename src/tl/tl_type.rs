use serde::{Deserialize, Serialize};
use crate::tl::tl_constructor::TlConstructor;


/// an abstract object, it doesn't exist really, it is there to categorize
#[derive(Serialize, Deserialize, Debug)]
pub struct TlType {
    pub name: String,
    pub constructors: Vec<TlConstructor>,
    //todo
    //pub common_parameters: Vec<TlParameter>,
}