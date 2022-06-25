use std::{
    cmp::Ordering,
    fmt::{write, Display},
};

use crate::issuer::Issuer;
use anyhow::Result;
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
        } else {
            return Err(anyhow::anyhow!("type {} not found", expr));
        }
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

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.id())
    }
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
    use crate::{
        type_env::default_env,
        types::{Type, TypeEq},
    };

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
