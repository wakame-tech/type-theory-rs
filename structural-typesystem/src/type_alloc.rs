use crate::{
    issuer::Issuer,
    types::{record_type, Id, Type, TypeExpr},
};
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use symbolic_expressions::Sexp;

/// [TypeAlloc] is globally unique.
#[derive(Debug, Clone)]
pub struct TypeAlloc {
    alloc: Vec<Type>,
}

impl Default for TypeAlloc {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeAlloc {
    pub fn new() -> Self {
        Self { alloc: vec![] }
    }

    /// create a new type variable
    pub fn new_variable(&mut self) -> Id {
        let id = self.alloc.len();
        self.alloc.push(Type::Variable { id, instance: None });
        id
    }

    pub fn new_operator(&mut self, name: &str, ids: &Vec<Id>) -> Id {
        let id = self.alloc.len();
        self.alloc.push(Type::Operator {
            id,
            name: name.to_string(),
            types: ids.to_vec(),
        });
        id
    }

    pub fn new_function(&mut self, arg: Id, ret: Id) -> Id {
        let id = self.alloc.len();
        let typ = Type::Operator {
            id,
            name: "->".to_string(),
            types: vec![arg, ret],
        };
        self.alloc.push(typ);
        id
    }

    pub fn new_primitive(&mut self, name: &str) -> Id {
        let id = self.alloc.len();
        let typ = Type::Operator {
            id,
            name: name.to_string(),
            types: vec![],
        };
        self.alloc.push(typ);
        id
    }

    pub fn new_record(&mut self, record: BTreeMap<String, Id>) -> Id {
        let id = self.alloc.len();
        let typ = Type::Record { id, types: record };
        self.alloc.push(typ);
        id
    }

    pub fn from_id(&self, id: Id) -> Result<Type> {
        self.alloc
            .get(id)
            .cloned()
            .ok_or(anyhow!("type_alloc type_id {} not found", id))
    }

    pub fn from_id_mut(&mut self, id: Id) -> Result<&mut Type> {
        self.alloc
            .get_mut(id)
            .ok_or(anyhow!("type_alloc type_id {} not found", id))
    }

    pub fn as_sexp(&self, type_id: Id, issuer: &mut Issuer) -> Result<TypeExpr> {
        issuer.count += 1;
        if issuer.count > 10 {
            return Err(anyhow!("cyclic type"));
        }
        match self.from_id(type_id)? {
            Type::Variable {
                instance: Some(inst),
                ..
            } => self.as_sexp(inst, issuer),
            Type::Variable { id, .. } => Ok(Sexp::String(issuer.name(id))),
            Type::Operator {
                ref types,
                ref name,
                ..
            } => {
                let types = types
                    .iter()
                    .map(|t| self.as_sexp(*t, issuer))
                    .collect::<Result<Vec<_>>>()?;
                if types.is_empty() {
                    Ok(Sexp::String(name.to_string()))
                } else {
                    Ok(Sexp::List(
                        vec![Sexp::String(name.to_string())]
                            .into_iter()
                            .chain(types)
                            .collect::<Vec<_>>(),
                    ))
                }
            }
            Type::Record { types, .. } => record_type(self, types),
        }
    }

    /// type parser
    pub fn from_sexp(&self, type_sexp: &TypeExpr) -> Result<Id> {
        let typ = self
            .alloc
            .iter()
            .find(|ty| self.as_sexp(ty.id(), &mut Default::default()).unwrap() == *type_sexp)
            .ok_or(anyhow!("type of \"{}\" not found", type_sexp))?;
        Ok(typ.id())
    }

    pub fn is_generic(&self, id: Id) -> Result<bool> {
        match self.from_id(id)? {
            Type::Operator { types, .. } => Ok(types.iter().any(|t| self.is_generic(*t).unwrap())),
            Type::Variable { .. } => Ok(true),
            Type::Record { types, .. } => Ok(types.values().any(|t| self.is_generic(*t).unwrap())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::type_env::TypeEnv;
    use anyhow::Result;
    use symbolic_expressions::parser::parse_str;

    #[test]
    fn parse_fn_type() -> Result<()> {
        let mut type_env = TypeEnv::default();
        let int_int = type_env.new_type(&parse_str("(-> int int)")?)?;
        assert_eq!(
            type_env.alloc.as_sexp(int_int, &mut Default::default())?,
            parse_str("(-> int int)")?,
        );
        Ok(())
    }

    #[test]
    fn parse_record_type() -> Result<()> {
        let mut type_env = TypeEnv::default();
        let rec = type_env.new_type(&parse_str("(record (a int))")?)?;
        assert_eq!(
            type_env.alloc.as_sexp(rec, &mut Default::default())?,
            parse_str("(record (a int))")?,
        );
        Ok(())
    }
}
