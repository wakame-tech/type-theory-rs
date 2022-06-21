use std::{cmp::Ordering, collections::HashMap};

use crate::issuer::{new_function, new_variable, Issuer};
use anyhow::{anyhow, Result};
pub type Id = usize;

#[derive(Debug, Clone, Hash)]
pub enum Type {
    Variable {
        id: Id,
        instance: Option<Id>,
    },
    /// ex. function type "->", tuple type ","
    Operator {
        id: Id,
        name: String,
        types: Vec<Id>,
    },
}

impl Type {
    /// type parser
    pub fn from(alloc: &Vec<Type>, expr: &str) -> Result<Type> {
        // TODO: composite type parser
        if let Some(typ) = alloc.iter().find(|ty| match ty {
            Type::Operator {
                id: _,
                name,
                types: _,
            } => name == expr,
            _ => todo!(),
        }) {
            return Ok(typ.clone());
        }
        todo!()
    }

    pub fn var(id: Id) -> Type {
        Type::Variable { id, instance: None }
    }

    pub fn fun(id: Id, arg: Id, ret: Id) -> Type {
        Type::Operator {
            id,
            name: "->".to_string(),
            types: vec![arg, ret],
        }
    }

    pub fn op(id: Id, name: &str, types: &[Id]) -> Type {
        Type::Operator {
            id,
            name: name.to_string(),
            types: types.to_vec(),
        }
    }

    pub fn id(&self) -> Id {
        match self {
            Type::Variable { id, .. } => *id,
            Type::Operator { id, .. } => *id,
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

    pub fn as_string(&self, a: &Vec<Type>, issuer: &mut Issuer) -> String {
        match self {
            &Type::Variable {
                instance: Some(inst),
                ..
            } => a[inst].as_string(a, issuer),
            &Type::Variable { .. } => issuer.name(self.id()),
            &Type::Operator {
                ref types,
                ref name,
                ..
            } => match types.len() {
                0 => name.clone(),
                2 => {
                    let l = a[types[0]].as_string(a, issuer);
                    let r = a[types[1]].as_string(a, issuer);
                    format!("({} {} {})", l, name, r)
                }
                _ => {
                    let mut coll = vec![];
                    for v in types {
                        coll.push(a[*v].as_string(a, issuer));
                    }
                    format!("{} {}", name, coll.join(" "))
                }
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Env(pub HashMap<String, Id>);

pub fn default_env() -> (Vec<Type>, Env) {
    // TODO: type hierarchy
    let mut alloc = vec![Type::op(0, "int", &[]), Type::op(1, "bool", &[])];
    let a = new_variable(&mut alloc);
    let env = Env(HashMap::from([
        ("true".to_string(), 1),
        ("false".to_string(), 1),
        ("not".to_string(), new_function(&mut alloc, 1, 1)),
        ("id".to_string(), new_function(&mut alloc, a, a)),
        ("zero?".to_string(), new_function(&mut alloc, 0, 1)),
        ("succ".to_string(), new_function(&mut alloc, 0, 0)),
    ]));
    (alloc, env)
}

pub trait TypeOrd {
    fn cmp(&self, other: &Type) -> Ordering;
}

pub trait TypeEq {
    fn eq(&self, other: &Type) -> bool;
}

impl TypeEq for Type {
    fn eq(&self, other: &Type) -> bool {
        match (self, other) {
            (Type::Operator { id: l, .. }, Type::Operator { id: r, .. }) => l == r,
            _ => todo!(),
        }
    }
}

impl TypeOrd for Type {
    fn cmp(&self, other: &Type) -> Ordering {
        match (self, other) {
            (Type::Operator { id: l, .. }, Type::Operator { id: r, .. }) => {
                if l == r {
                    return Ordering::Equal;
                }
                todo!()
            }
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;

    use crate::types::TypeEq;

    use super::{default_env, Type, TypeOrd};
    use anyhow::Result;

    #[test]
    fn test_bool_eq() -> Result<()> {
        let (alloc, env) = default_env();
        let t1 = Type::from(&alloc, "bool")?;
        let t2 = Type::from(&alloc, "bool")?;
        assert_eq!(t1.eq(&t2), true);
        Ok(())
    }

    #[test]
    fn test_bool_neq() -> Result<()> {
        let (alloc, env) = default_env();
        let t1 = Type::from(&alloc, "int")?;
        let t2 = Type::from(&alloc, "bool")?;
        assert_eq!(t1.eq(&t2), false);
        Ok(())
    }
}
