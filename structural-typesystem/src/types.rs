use crate::type_alloc::TypeAlloc;
use anyhow::Result;
use std::collections::BTreeMap;
use symbolic_expressions::Sexp;

pub type Id = usize;
pub type TypeExpr = Sexp;

pub fn record_type(alloc: &TypeAlloc, types: BTreeMap<String, Id>) -> Result<TypeExpr> {
    Ok(Sexp::List(
        vec![Ok(Sexp::String("record".to_string()))]
            .into_iter()
            .chain(types.into_iter().map(|(k, v)| {
                let ty = alloc.as_sexp(v, &mut Default::default())?;
                Ok(Sexp::List(vec![Sexp::String(k.to_string()), ty]))
            }))
            .collect::<Result<_>>()?,
    ))
}

#[derive(Debug, Clone, Hash)]
pub enum Type {
    Variable {
        id: Id,
        instance: Option<Id>,
    },
    /// - function type "->"
    /// - apply type "app"
    /// - tuple type ","
    Operator {
        id: Id,
        name: String,
        types: Vec<Id>,
    },
    Record {
        id: Id,
        types: BTreeMap<String, Id>,
    },
}

impl Type {
    pub fn id(&self) -> Id {
        match self {
            Type::Variable { id, .. } => *id,
            Type::Operator { id, .. } => *id,
            Type::Record { id, .. } => *id,
        }
    }

    pub fn set_instance(&mut self, id: Id) {
        match self {
            Type::Variable { instance, .. } => {
                *instance = Some(id);
            }
            _ => panic!("set_instance called on non-variable type"),
        }
    }
}
