use std::collections::HashMap;

use crate::types::{Id, Type};

pub struct Issuer {
    pub value: u8,
    pub set: HashMap<Id, String>,
}

impl Issuer {
    pub fn new(start: char) -> Self {
        Issuer {
            value: start as u8,
            set: HashMap::new(),
        }
    }

    fn next(&mut self) -> String {
        let id = self.value;
        self.value += 1;
        format!("{}", id as char)
    }

    /// get or create
    pub fn name(&mut self, id: Id) -> String {
        if let Some(name) = self.set.get(&id) {
            name.clone()
        } else {
            let name = self.next();
            self.set.insert(id, name.clone());
            name
        }
    }
}

pub fn new_variable(a: &mut Vec<Type>) -> Id {
    let id = a.len();
    a.push(Type::var(id));
    id
}

pub fn new_function(a: &mut Vec<Type>, arg: Id, ret: Id) -> Id {
    let id = a.len();
    let typ = Type::op(id, "->", &[arg, ret]);
    a.push(typ);
    id
}

pub fn new_operator(a: &mut Vec<Type>, name: &str, types: &[Id]) -> Id {
    let id = a.len();
    let typ = Type::op(id, name, types);
    a.push(typ);
    id
}
