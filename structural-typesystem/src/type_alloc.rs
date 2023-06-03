use crate::{
    issuer::Issuer,
    types::{Id, Type},
};
use anyhow::{anyhow, Result};
use symbolic_expressions::{parser::parse_str, Sexp};

/// [TypeAlloc] is globally unique.
#[derive(Debug, Clone)]
pub struct TypeAlloc {
    pub alloc: Vec<Type>,
}

impl Default for TypeAlloc {
    fn default() -> Self {
        let mut res = Self { alloc: vec![] };
        register_builtin_types(&mut res);
        res
    }
}

impl TypeAlloc {
    /// create a new type variable
    pub fn new_variable(&mut self) -> Id {
        let id = self.alloc.len();
        self.alloc.push(Type::var(id));
        id
    }

    /// create a new operator type
    pub fn new_operator(&mut self, name: &str, types: &Vec<Id>) -> Id {
        println!("new_operator: {} = {:?}", name, types);
        let id = self.alloc.len();
        self.alloc.push(Type::Operator {
            id,
            name: name.to_string(),
            types: types.to_vec(),
        });
        id
    }

    /// create a new function type
    ///
    /// ```
    /// new_function(&mut alloc, 0, 0);
    /// ```
    pub fn new_function(&mut self, arg: Id, ret: Id) -> Id {
        let id = self.alloc.len();
        let typ = Type::Operator {
            id,
            name: "->".to_string(),
            types: vec![arg, ret],
        };
        self.alloc.push(typ);
        println!("new_function: {} = {} -> {}", id, arg, ret);
        id
    }

    /// create a new primitive type
    pub fn new_primitive(&mut self, name: &str) -> Id {
        println!("new_primitive: {}", name);
        let id = self.alloc.len();
        let typ = Type::Operator {
            id,
            name: name.to_string(),
            types: vec![],
        };
        self.alloc.push(typ);
        id
    }

    pub fn from_id(&self, id: Id) -> Option<Type> {
        self.alloc.get(id).map(|t| t.clone())
    }

    pub fn as_sexp(&self, type_id: Id, issuer: &mut Issuer) -> Result<Sexp> {
        let Some(typ) = self.from_id(type_id) else {
            return Err(anyhow::anyhow!("type_id {} not found", type_id));
        };
        match typ {
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
            Type::Record { .. } => todo!(),
        }
    }

    pub fn as_string(&self, type_id: Id, issuer: &mut Issuer) -> Result<String> {
        self.as_sexp(type_id, issuer).map(|sexp| sexp.to_string())
    }

    /// type parser
    pub fn from_sexp(&self, type_sexp: &Sexp) -> Result<Id> {
        let typ = self
            .alloc
            .iter()
            .find(|ty| self.as_sexp(ty.id(), &mut Default::default()).unwrap() == *type_sexp)
            .ok_or(anyhow!("type {} not found", type_sexp))?;
        Ok(typ.id())
    }

    pub fn from(&self, expr: &str) -> Result<Id> {
        let type_sexp = parse_str(expr)?;
        self.from_sexp(&type_sexp)
    }
}

/// register builtin types
pub fn register_builtin_types(alloc: &mut TypeAlloc) {
    alloc.new_primitive("any");
    alloc.new_primitive("bool");
    alloc.new_primitive("int");
}

#[cfg(test)]
mod tests {
    use super::TypeAlloc;
    use anyhow::Result;

    #[test]
    fn parse_fn_type() -> Result<()> {
        let mut alloc = TypeAlloc::default();
        let int_type_id = alloc.from("int")?;
        alloc.new_function(int_type_id, int_type_id);

        let ty = alloc.from("(-> int int)")?;
        assert_eq!(
            alloc.as_string(ty, &mut Default::default())?,
            "(-> int int)"
        );
        Ok(())
    }
}
