use std::collections::BTreeMap;
use symbolic_expressions::Sexp;

pub type Id = usize;
pub type TypeExpr = Sexp;

pub const RECORD_TYPE_KEYWORD: &str = "record";
pub const FN_TYPE_KEYWORD: &str = "->";

#[derive(Debug, Clone, Hash)]
pub enum Type {
    Primitive {
        id: Id,
        name: String,
    },
    Variable {
        id: Id,
        instance: Option<Id>,
    },
    Function {
        id: Id,
        args: Vec<Id>,
        ret: Id,
    },
    Record {
        id: Id,
        fields: BTreeMap<String, Id>,
    },
}

impl Type {
    pub fn id(&self) -> Id {
        match self {
            Type::Primitive { id, .. } => *id,
            Type::Variable { id, .. } => *id,
            Type::Function { id, .. } => *id,
            Type::Record { id, .. } => *id,
        }
    }

    pub fn primitive(id: Id, name: &str) -> Self {
        Type::Primitive {
            id,
            name: name.to_string(),
        }
    }

    pub fn variable(id: Id) -> Self {
        Type::Variable { id, instance: None }
    }

    pub fn set_instance(&mut self, id: Id) {
        match self {
            Type::Variable { instance, .. } => {
                *instance = Some(id);
            }
            _ => panic!("set_instance called on non-variable type"),
        }
    }

    pub fn function(id: Id, args: Vec<Id>, ret: Id) -> Self {
        Type::Function { id, args, ret }
    }

    pub fn record(id: Id, fields: BTreeMap<String, Id>) -> Self {
        Type::Record { id, fields }
    }
}
