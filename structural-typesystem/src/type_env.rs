use crate::{
    type_alloc::TypeAlloc,
    types::{
        Id, Type, TypeExpr, FN_TYPE_KEYWORD, GETTER_TYPE_KEYWORD, LIST_TYPE_KEYWORD,
        RECORD_TYPE_KEYWORD,
    },
};
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
    variables: HashMap<String, Id>,
    /// key is stringified sexp
    id_map: HashMap<String, Id>,
    /// subtyping tree
    index_map: HashMap<Id, NodeIndex>,
    tree: Graph<Id, ()>,
}

pub fn arrow(args: Vec<TypeExpr>, ret: TypeExpr) -> TypeExpr {
    Sexp::List(vec![
        Sexp::String(FN_TYPE_KEYWORD.to_string()),
        Sexp::List(args),
        ret,
    ])
}

pub fn record(fields: BTreeMap<String, TypeExpr>) -> TypeExpr {
    Sexp::List(
        vec![Sexp::String(RECORD_TYPE_KEYWORD.to_string())]
            .into_iter()
            .chain(fields.iter().map(|(k, v)| {
                Sexp::List(vec![
                    Sexp::String(k.to_string()),
                    Sexp::String(":".to_string()),
                    v.clone(),
                ])
            }))
            .collect(),
    )
}

pub fn container(name: String, elements: Vec<TypeExpr>) -> TypeExpr {
    Sexp::List(
        vec![Sexp::String(name)]
            .into_iter()
            .chain(elements)
            .collect(),
    )
}

impl Default for TypeEnv {
    fn default() -> Self {
        let mut env = TypeEnv::new();
        let any = env.new_type_str("any").unwrap();
        let int = env.new_type_str("int").unwrap();
        env.new_subtype(int, any);

        let bool = env.new_type_str("bool").unwrap();
        env.new_subtype(bool, any);

        let atom = env.new_type_str("atom").unwrap();
        env.new_subtype(atom, any);

        let vec = env.new_type_str("vec").unwrap();
        env.new_subtype(vec, any);

        env
    }
}

impl TypeEnv {
    pub fn new() -> Self {
        Self {
            alloc: TypeAlloc::new(),
            variables: HashMap::new(),
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
        self.alloc.as_sexp(id)
    }

    /// register `a` as subtype of `b`
    pub fn new_subtype(&mut self, a: Id, b: Id) {
        let (ai, bi) = (self.index_map[&a], self.index_map[&b]);
        self.tree.add_edge(bi, ai, ());
    }

    pub fn new_alias(&mut self, name: &str, ty: Id) {
        self.id_map.insert(name.to_string(), ty);
    }

    pub fn new_type(&mut self, ty: &TypeExpr) -> Result<Id> {
        if let Some(id) = self.id_map.get(&ty.to_string()) {
            return Ok(*id);
        }
        match ty {
            Sexp::String(v) if v.len() == 1 => {
                let id = self.alloc.issue_id();
                self.alloc.insert(Type::variable(id));
                self.register_type_id(ty, id);
                Ok(id)
            }
            Sexp::String(s) => {
                let id = self.alloc.issue_id();
                self.alloc.insert(Type::primitive(id, s));
                self.register_type_id(ty, id);

                // all atoms are subtypes of atom
                if s.starts_with(":") {
                    let atom_ty = self.new_type_str("atom")?;
                    self.new_subtype(id, atom_ty);
                }

                Ok(id)
            }
            Sexp::List(list) if list[0].string()? == FN_TYPE_KEYWORD => {
                let args = list[1]
                    .list()?
                    .iter()
                    .map(|s| self.new_type(s))
                    .collect::<Result<Vec<_>>>()?;
                let ret = self.new_type(&list[2])?;
                let id = self.alloc.issue_id();
                self.alloc.insert(Type::function(id, args, ret));
                self.register_type_id(ty, id);
                Ok(id)
            }
            Sexp::List(list) if list[0].string()? == RECORD_TYPE_KEYWORD => {
                let fields = list[1..]
                    .iter()
                    .map(|s| -> Result<_> {
                        let l = s.list()?;
                        let k = &l[0].string()?;
                        anyhow::ensure!(l[1].string()? == ":", "missing colon {:?}", l);
                        let v = &l[2];
                        let id = self.new_type(v)?;
                        Ok((k.to_string(), id))
                    })
                    .collect::<Result<BTreeMap<_, _>>>()?;
                let id = self.alloc.issue_id();
                self.alloc.insert(Type::record(id, fields));
                self.register_type_id(ty, id);
                Ok(id)
            }
            Sexp::List(list) if list[0].string()? == LIST_TYPE_KEYWORD => {
                let elements = list[1..]
                    .iter()
                    .map(|s| self.new_type(s))
                    .collect::<Result<Vec<_>>>()?;
                let con = self.new_type(&list[0])?;
                let id = self.alloc.issue_id();
                self.alloc.insert(Type::container(con, elements));
                self.register_type_id(ty, id);
                Ok(id)
            }
            // ([] t k)
            Sexp::List(list) if list[0].string()? == GETTER_TYPE_KEYWORD => {
                // t : record
                let container_ty = self.new_type(&list[1])?;
                let key_ty = self.new_type(&list[2])?;

                if !matches!(self.alloc.get(container_ty)?, Type::Record { .. }) {
                    return Err(anyhow::anyhow!("{} is not record", &list[1]));
                }
                let atom_ty = self.new_type_str("atom")?;
                if !self.is_subtype(key_ty, atom_ty)? {
                    return Err(anyhow::anyhow!("{} is not subtype of atom", &list[2]));
                }

                let id = self.alloc.issue_id();
                self.alloc
                    .insert(Type::container(id, vec![container_ty, key_ty]));
                self.register_type_id(ty, id);
                Ok(id)
            }
            _ => Err(anyhow::anyhow!(
                "TypeEnv::new_type() unsupported type: {}",
                ty
            )),
        }
    }

    pub fn new_type_str(&mut self, ty: &str) -> Result<Id> {
        self.new_type(&parse_str(ty)?)
    }

    pub fn set_variable(&mut self, name: &str, ty: Id) {
        self.variables.insert(name.to_string(), ty);
    }

    pub fn get_variable(&self, name: &str) -> Result<Id> {
        self.variables
            .get(name)
            .cloned()
            .ok_or(anyhow::anyhow!("{} not found", name))
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
