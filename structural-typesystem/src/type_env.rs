use crate::{
    type_alloc::TypeAlloc,
    types::{Id, TypeExpr, FN_TYPE_KEYWORD, RECORD_TYPE_KEYWORD},
};
use anyhow::Result;
use petgraph::prelude::*;
use std::{
    collections::{BTreeMap, HashMap},
    fmt::{Debug, Display},
};
use symbolic_expressions::Sexp;

/// [TypeEnv] will be created per each [Expr::FnDef]
#[derive(Debug, Clone)]
pub struct TypeEnv {
    pub alloc: TypeAlloc,
    /// key is stringified sexp
    id_map: HashMap<String, Id>,
    /// subtyping tree
    index_map: HashMap<Id, NodeIndex>,
    tree: Graph<Id, ()>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self {
            alloc: TypeAlloc::new(),
            id_map: HashMap::new(),
            index_map: HashMap::new(),
            tree: Graph::new(),
        }
    }

    pub fn get(&self, type_expr: &TypeExpr) -> Result<Id> {
        self.id_map
            .get(type_expr.to_string().as_str())
            .cloned()
            .ok_or(anyhow::anyhow!("{} not found", type_expr))
    }

    pub fn get_by_id(&self, id: &Id) -> Result<String> {
        self.id_map
            .iter()
            .find(|(_, v)| **v == *id)
            .map(|(k, _)| k.clone())
            .ok_or(anyhow::anyhow!("{} not found", id))
    }

    pub fn type_name(&self, id: Id) -> Result<Sexp> {
        self.alloc.as_sexp(id, &mut Default::default())
    }

    /// register `a` as subtype of `b`
    pub fn new_subtype(&mut self, a: Id, b: Id) {
        let (ai, bi) = (self.index_map[&a], self.index_map[&b]);
        self.tree.add_edge(bi, ai, ());
    }

    pub fn new_type(&mut self, type_expr: &TypeExpr) -> Result<Id> {
        if let Some(id) = self.id_map.get(&type_expr.to_string()) {
            return Ok(*id);
        }
        match type_expr {
            Sexp::String(v) if v.len() == 1 => {
                let id = self.alloc.new_variable();
                self.register_type_id(type_expr, id);
                Ok(id)
            }
            Sexp::String(s) => {
                let id = self.alloc.new_primitive(s);
                self.register_type_id(type_expr, id);
                Ok(id)
            }
            Sexp::List(list) if list[0].string()? == FN_TYPE_KEYWORD => {
                let (f, t) = (self.new_type(&list[1])?, self.new_type(&list[2])?);
                let id = self.alloc.new_function(f, t);
                self.register_type_id(type_expr, id);
                Ok(id)
            }
            Sexp::List(list) if list[0].string()? == RECORD_TYPE_KEYWORD => {
                let entries = list[1..]
                    .iter()
                    .map(|s| -> Result<_> {
                        let l = s.list()?;
                        let (k, v) = (&l[0].string()?, &l[1]);
                        let id = self.new_type(v)?;
                        Ok((k.to_string(), id))
                    })
                    .collect::<Result<Vec<_>>>()?;
                let id = self.alloc.new_record(BTreeMap::from_iter(entries));
                self.register_type_id(type_expr, id);
                Ok(id)
            }
            _ => panic!(),
        }
    }

    pub fn new_type_from_id(&mut self, id: Id) -> Result<()> {
        let expr = self.alloc.as_sexp(id, &mut Default::default())?;
        self.register_type_id(&expr, id);
        Ok(())
    }

    fn register_type_id(&mut self, expr: &TypeExpr, type_id: Id) {
        self.id_map.insert(expr.to_string(), type_id);
        let i = self.tree.add_node(type_id);
        self.index_map.insert(type_id, i);
    }

    /// returns true is a is subtype of b
    pub fn has_edge(&self, a: Id, b: Id) -> bool {
        let (ai, bi) = (self.index_map[&a], self.index_map[&b]);
        a == b || self.tree.find_edge(bi, ai).is_some()
    }
}

impl Display for TypeEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (name, ty_id) in &self.id_map {
            writeln!(f, "#{}: {}", ty_id, name)?;
        }
        Ok(())
    }
}
