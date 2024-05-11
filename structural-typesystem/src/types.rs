use std::collections::BTreeMap;
use symbolic_expressions::Sexp;

pub type Id = usize;
pub type TypeExpr = Sexp;

pub const RECORD_TYPE_KEYWORD: &'static str = "record";
pub const FN_TYPE_KEYWORD: &'static str = "->";

#[derive(Debug, Clone, Hash)]
pub enum Type {
    Variable {
        id: Id,
        instance: Option<Id>,
    },
    Operator {
        id: Id,
        op: String,
        types: BTreeMap<Option<String>, Id>,
    },
}

impl Type {
    pub fn id(&self) -> Id {
        match self {
            Type::Variable { id, .. } => *id,
            Type::Operator { id, .. } => *id,
        }
    }

    pub fn primitive(id: Id, name: &str) -> Self {
        Type::Operator {
            id,
            op: name.to_string(),
            types: BTreeMap::new(),
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

    pub fn function(id: Id, arg: Id, ret: Id) -> Self {
        Type::Operator {
            id,
            op: "->".to_string(),
            types: BTreeMap::from_iter(vec![(None, arg), (None, ret)]),
        }
    }

    pub fn record(id: Id, record: BTreeMap<String, Id>) -> Self {
        Type::Operator {
            id,
            op: "record".to_string(),
            types: record.into_iter().map(|(k, v)| (Some(k), v)).collect(),
        }
    }
}
