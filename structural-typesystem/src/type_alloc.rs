use crate::{
    issuer::Issuer,
    types::{Id, Type, TypeExpr},
};
use anyhow::{anyhow, Result};
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

    pub fn get(&self, id: Id) -> Result<Type> {
        self.alloc
            .get(id)
            .cloned()
            .ok_or(anyhow!("type_alloc type_id {} not found", id))
    }

    pub fn get_mut(&mut self, id: Id) -> Result<&mut Type> {
        self.alloc
            .get_mut(id)
            .ok_or(anyhow!("type_alloc type_id {} not found", id))
    }

    pub fn issue_id(&self) -> Id {
        self.alloc.len()
    }

    pub fn insert(&mut self, ty: Type) {
        self.alloc.push(ty);
    }

    pub fn as_sexp(&self, id: Id) -> Result<TypeExpr> {
        self.as_sexp_rec(id, &mut Default::default(), 0)
    }

    fn as_sexp_rec(&self, id: Id, issuer: &mut Issuer, nest: usize) -> Result<TypeExpr> {
        if nest > 10 {
            return Err(anyhow!("cyclic type"));
        }
        match self.get(id)? {
            // primitive types
            Type::Primitive { name, .. } => Ok(Sexp::String(name)),
            // concrete types
            Type::Variable {
                instance: Some(inst),
                ..
            } => self.as_sexp_rec(inst, issuer, nest + 1),
            // type variables
            Type::Variable { id, .. } => Ok(Sexp::String(issuer.name(id))),
            Type::Function { arg, ret, .. } => Ok(Sexp::List(vec![
                Sexp::String("->".to_string()),
                self.as_sexp_rec(arg, issuer, nest + 1)?,
                self.as_sexp_rec(ret, issuer, nest + 1)?,
            ])),
            Type::Record { fields, .. } => {
                let fields = fields
                    .iter()
                    .map(|(label, id)| {
                        Ok(Sexp::List(vec![
                            Sexp::String(label.to_string()),
                            Sexp::String(":".to_string()),
                            self.as_sexp_rec(*id, issuer, nest + 1)?,
                        ]))
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(Sexp::List(
                    vec![Sexp::String("record".to_string())]
                        .into_iter()
                        .chain(fields)
                        .collect::<Vec<_>>(),
                ))
            }
        }
    }

    /// type parser
    pub fn from_sexp(&self, type_sexp: &TypeExpr) -> Result<Id> {
        let typ = self
            .alloc
            .iter()
            .find(|ty| self.as_sexp(ty.id()).unwrap() == *type_sexp)
            .ok_or(anyhow!("type of \"{}\" not found", type_sexp))?;
        Ok(typ.id())
    }

    pub fn is_generic(&self, id: Id) -> Result<bool> {
        match self.get(id)? {
            Type::Function { arg, ret, .. } => Ok(self.is_generic(arg)? || self.is_generic(ret)?),
            Type::Record { fields, .. } => {
                Ok(fields.values().any(|id| self.is_generic(*id).unwrap()))
            }
            Type::Primitive { .. } => Ok(false),
            Type::Variable { .. } => Ok(true),
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
        assert_eq!(type_env.alloc.as_sexp(int_int)?, parse_str("(-> int int)")?,);
        Ok(())
    }

    #[test]
    fn parse_record_type() -> Result<()> {
        let mut type_env = TypeEnv::default();
        let rec = type_env.new_type(&parse_str("(record (a : int))")?)?;
        assert_eq!(
            type_env.alloc.as_sexp(rec)?,
            parse_str("(record (a : int))")?,
        );
        Ok(())
    }
}
