use crate::{type_alloc::TypeAlloc, types::Id};
use anyhow::Result;
use petgraph::prelude::*;
use ptree::graph::print_graph;
use std::{collections::HashMap, fmt::Debug};

/// [TypeEnv] will be created per each [Expr::FnDef]
#[derive(Debug, Clone)]
pub struct TypeEnv {
    pub id_map: HashMap<String, Id>,
    index_map: HashMap<Id, NodeIndex>,
    tree: Graph<Id, ()>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self {
            id_map: HashMap::new(),
            index_map: HashMap::new(),
            tree: Graph::new(),
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
    pub fn is_subtype(&self, a: Id, b: Id) -> bool {
        let (ai, bi) = (self.index_map[&a], self.index_map[&b]);
        a == b || self.tree.find_edge(bi, ai).is_some()
    }

    pub fn get_id(&self, name: &str) -> Option<Id> {
        self.id_map.get(name).cloned()
    }

    pub fn debug(&self, alloc: &TypeAlloc) -> Result<()> {
        let any = alloc.from("any")?;
        print_graph(&self.tree, self.index_map[&any])?;
        Ok(())
    }
}
