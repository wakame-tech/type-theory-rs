use crate::{
    type_alloc::TypeAlloc,
    types::{
        Id, Type, TypeExpr, FN_TYPE_KEYWORD, GETTER_TYPE_KEYWORD, LIST_TYPE_KEYWORD,
        RECORD_TYPE_KEYWORD, SUBTYPE_KEYWORD, UNION_TYPE_KEYWORD,
    },
};
use anyhow::Result;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
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
        env.new_type_str("any").unwrap();
        env.new_type_str("int").unwrap();
        env.new_type_str("bool").unwrap();
        env.new_type_str("atom").unwrap();
        env.new_type_str("str").unwrap();
        env.new_type_str("vec").unwrap();
        env
    }
}

impl TypeEnv {
    pub fn new() -> Self {
        Self {
            alloc: TypeAlloc::new(),
            variables: HashMap::new(),
            id_map: HashMap::new(),
        }
    }

    pub fn get(&self, type_expr: &TypeExpr) -> Result<Id> {
        self.id_map
            .get(type_expr.to_string().as_str())
            .cloned()
            .ok_or(anyhow::anyhow!("{} not found", type_expr))
    }

    pub fn get_str(&self, ty: &str) -> Result<Id> {
        self.get(&parse_str(ty)?)
    }

    /// union of `types`. a single type is returned as-is
    pub fn new_union(&mut self, types: BTreeSet<Id>) -> Result<Id> {
        if types.len() == 1 {
            return Ok(*types.first().unwrap());
        }
        if types
            .iter()
            .any(|t| self.alloc.is_generic(*t).unwrap_or(true))
        {
            // type variables cannot round-trip through their names
            let id = self.alloc.issue_id();
            self.alloc.insert(Type::Union { id, types });
            return Ok(id);
        }
        let types = types
            .iter()
            .map(|t| self.type_name(*t))
            .collect::<Result<Vec<_>>>()?;
        self.new_type(&container(UNION_TYPE_KEYWORD.to_string(), types))
    }

    pub fn type_name(&self, id: Id) -> Result<Sexp> {
        self.alloc.as_sexp(id)
    }

    pub fn new_alias(&mut self, name: &str, ty: Id) {
        self.id_map.insert(name.to_string(), ty);
    }

    pub fn new_type(&mut self, ty: &TypeExpr) -> Result<Id> {
        if let Some(id) = self.id_map.get(&ty.to_string()) {
            return Ok(*id);
        }

        let id = match ty {
            Sexp::String(v) if v.len() == 1 && v.chars().next().unwrap().is_alphabetic() => {
                self.insert(|id| Type::variable(id, None))
            }
            Sexp::String(s) => self.insert(|id| Type::primitive(id, s)),
            Sexp::List(list)
                if list.len() == 3
                    && list[1].is_string()
                    && list[1].string()? == FN_TYPE_KEYWORD =>
            {
                let args = list[0]
                    .list()?
                    .iter()
                    .map(|s| self.new_type(s))
                    .collect::<Result<Vec<_>>>()?;
                let ret = self.new_type(&list[2])?;
                self.insert(|id| Type::function(id, args, ret))
            }
            Sexp::List(list) if list[0].string()? == RECORD_TYPE_KEYWORD => {
                let fields = list[1..]
                    .iter()
                    .map(|s| -> Result<_> {
                        let l = s.list()?;
                        let k = &l[0].string()?;
                        anyhow::ensure!(l[1].string()? == ":", "missing colon {:?}", l);
                        let id = self.new_type(&l[2])?;
                        Ok((k.to_string(), id))
                    })
                    .collect::<Result<BTreeMap<_, _>>>()?;
                self.insert(|id| Type::record(id, fields))
            }
            // (vec a), ([] a b)
            Sexp::List(list)
                if list[0].string()? == LIST_TYPE_KEYWORD
                    || (list.len() == 3 && list[0].string()? == GETTER_TYPE_KEYWORD) =>
            {
                let con = self.new_type(&list[0])?;
                let elements = list[1..]
                    .iter()
                    .map(|s| self.new_type(s))
                    .collect::<Result<Vec<_>>>()?;
                // the id field of a container points to its constructor
                self.insert(|_| Type::container(con, elements))
            }
            Sexp::List(list) if list[0].string()? == UNION_TYPE_KEYWORD => {
                let types = list[1..]
                    .iter()
                    .map(|s| self.new_type(s))
                    .collect::<Result<BTreeSet<_>>>()?;
                self.insert(|id| Type::Union { id, types })
            }
            Sexp::List(list)
                if list.len() == 3
                    && list[1].is_string()
                    && list[1].string()? == SUBTYPE_KEYWORD =>
            {
                let is_type_var = list[0].is_string()
                    && list[0].string()?.chars().next().unwrap().is_alphabetic();
                if !is_type_var {
                    return Err(anyhow::anyhow!("must be type variable: {:?}", list[0]));
                }
                let upper_bound = self.new_type(&list[2])?;
                self.insert(|id| Type::variable(id, Some(upper_bound)))
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "TypeEnv::new_type() unsupported type: {}",
                    ty
                ))
            }
        };
        log::debug!("new_type: {} #{}", ty, id);
        self.register_type_id(ty, id);
        Ok(id)
    }

    /// allocates a new type built from its id
    fn insert(&mut self, f: impl FnOnce(Id) -> Type) -> Id {
        let id = self.alloc.issue_id();
        self.alloc.insert(f(id));
        id
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
