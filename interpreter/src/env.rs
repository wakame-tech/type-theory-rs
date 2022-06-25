use std::collections::HashMap;

use structural_typesystem::types::Type;
use symbolic_expressions::Sexp;

type Value = i64;

#[derive(Debug)]
pub struct Env {
    pub alloc: Vec<Type>,
    pub variables: HashMap<String, Value>,
    pub functions: HashMap<String, (String, Vec<Sexp>)>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            alloc: Vec::new(),
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }
}
