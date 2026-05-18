use std::collections::HashMap;
use crate::modules::types::{MangledName, WindValueID};

#[derive(Debug)]
pub struct Bindings {
    pub name_to_value: HashMap<MangledName, WindValueID>,
    pub value_to_names: HashMap<WindValueID, Vec<MangledName>>,
}

