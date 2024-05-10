use crate::type_alloc::TypeAlloc;
use anyhow::Result;
use std::collections::BTreeMap;
use symbolic_expressions::Sexp;

pub type Id = usize;
pub type TypeExpr = Sexp;

pub const RECORD_TYPE_KEYWORD: &'static str = "record";
pub const FN_TYPE_KEYWORD: &'static str = "->";

pub fn record_type(alloc: &TypeAlloc, types: BTreeMap<String, Id>) -> Result<TypeExpr> {
    Ok(Sexp::List(
        vec![Ok(Sexp::String(RECORD_TYPE_KEYWORD.to_string()))]
            .into_iter()
            .chain(types.into_iter().map(|(k, v)| {
                let ty = alloc.as_sexp(v, &mut Default::default())?;
                Ok(Sexp::List(vec![Sexp::String(k.to_string()), ty]))
            }))
            .collect::<Result<_>>()?,
    ))
}

pub fn fn_type(alloc: &TypeAlloc, arg: Id, ret: Id) -> Result<TypeExpr> {
    Ok(Sexp::List(vec![
        Sexp::String(FN_TYPE_KEYWORD.to_string()),
        alloc.as_sexp(arg, &mut Default::default())?,
        alloc.as_sexp(ret, &mut Default::default())?,
    ]))
}

#[derive(Debug, Clone, Hash)]
pub enum Type {
    Variable {
        id: Id,
        instance: Option<Id>,
    },
    /// function type "->"
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
