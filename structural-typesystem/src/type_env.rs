use std::collections::HashMap;

use crate::{
    issuer::{new_function, new_variable},
    types::{Id, Type},
};

#[derive(Debug, Clone)]
pub struct TypeEnv(pub HashMap<String, Id>);

pub fn default_env() -> (Vec<Type>, TypeEnv) {
    // TODO: type hierarchy
    let mut alloc = vec![Type::op(0, "int", &[]), Type::op(1, "bool", &[])];
    let a = new_variable(&mut alloc);
    let env = TypeEnv(HashMap::from([
        ("true".to_string(), 1),
        ("false".to_string(), 1),
        ("not".to_string(), new_function(&mut alloc, 1, 1)),
        ("id".to_string(), new_function(&mut alloc, a, a)),
        ("zero?".to_string(), new_function(&mut alloc, 0, 1)),
        ("succ".to_string(), new_function(&mut alloc, 0, 0)),
    ]));
    (alloc, env)
}
