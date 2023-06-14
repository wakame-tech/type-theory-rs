use crate::{type_alloc::TypeAlloc, types::Id};
use anyhow::Result;
use petgraph::prelude::*;
use std::{
    collections::{BTreeMap, HashMap},
    fmt::{Debug, Display},
};
use symbolic_expressions::{parser::parse_str, Sexp};

/// [TypeEnv] will be created per each [Expr::FnDef]
#[derive(Debug, Clone)]
pub struct TypeEnv {
    pub alloc: TypeAlloc,
    pub id_map: HashMap<String, Id>,
    index_map: HashMap<Id, NodeIndex>,
    tree: Graph<Id, ()>,
}

impl TypeEnv {
    pub fn new(alloc: TypeAlloc) -> Self {
        Self {
            alloc,
            id_map: HashMap::new(),
            index_map: HashMap::new(),
            tree: Graph::new(),
        }
    }

    pub fn get(&mut self, expr: &str) -> Result<Id> {
        dbg!(expr);
        if let Some(id) = self.id_map.get(expr) {
            Ok(*id)
        } else {
            match parse_str(expr)? {
                Sexp::String(s) if s == "*" => Ok(self.alloc.new_variable()),
                Sexp::String(s) => Ok(self.alloc.new_primitive(&s)),
                Sexp::List(list) if list[0].string()? == "->" => {
                    let (f, t) = (
                        self.get(&list[1].to_string())?,
                        self.get(&list[2].to_string())?,
                    );
                    Ok(self.alloc.new_function(f, t))
                }
                Sexp::List(list) if list[0].string()? == "record" => {
                    let entries = list[1..]
                        .iter()
                        .map(|s| -> Result<_> {
                            let l = s.list()?;
                            let (k, v) = (l[0].string()?, l[1].string()?);
                            let id = self.get(v)?;
                            Ok((k.to_string(), id))
                        })
                        .collect::<Result<Vec<_>>>()?;
                    Ok(self.alloc.new_record(BTreeMap::from_iter(entries)))
                }
                _ => panic!(),
            }
        }
    }

    pub fn add(&mut self, name: &str, type_id: Id) {
        self.id_map.insert(name.to_string(), type_id);
        let i = self.tree.add_node(type_id);
        self.index_map.insert(type_id, i);
    }

    /// register `a` as subtype of `b`
    pub fn subtype(&mut self, a: Id, b: Id) {
        let (ai, bi) = (self.index_map[&a], self.index_map[&b]);
        self.tree.add_edge(bi, ai, ());
    }

    /// returns true is a is subtype of b
    pub fn has_edge(&self, a: Id, b: Id) -> bool {
        let (ai, bi) = (self.index_map[&a], self.index_map[&b]);
        a == b || self.tree.find_edge(bi, ai).is_some()
    }

    pub fn get_id(&self, name: &str) -> Option<Id> {
        self.id_map.get(name).cloned()
    }
}

impl Display for TypeEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (k, typ) in &self.id_map {
            writeln!(
                f,
                "- {} :: {}",
                k,
                self.alloc.as_sexp(*typ, &mut Default::default()).unwrap()
            )?;
        }
        Ok(())
    }
}
