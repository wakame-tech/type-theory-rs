use crate::{type_alloc::TypeAlloc, type_env::TypeEnv, types::Type};
use std::cmp::Ordering;

pub trait TypeOrd {
    fn cmp(&self, env: &TypeEnv, alloc: &TypeAlloc, other: &Type) -> Ordering;
}

pub trait TypeEq {
    fn eq(&self, env: &TypeEnv, alloc: &TypeAlloc, other: &Type) -> bool;
}

impl TypeEq for Type {
    fn eq(&self, _: &TypeEnv, other: &Type) -> bool {
        match (self, other) {
            (Type::Operator { id: l, .. }, Type::Operator { id: r, .. }) => l == r,
            _ => todo!(),
        }
    }
}

impl TypeOrd for Type {
    fn cmp(&self, _: &TypeEnv, other: &Type) -> Ordering {
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
    use crate::{subtyping::TypeEq, type_env::setup_type_env};
    use anyhow::Result;

    #[test]
    fn test_bool_eq() -> Result<()> {
        let (mut env, mut alloc) = setup_type_env()?;
        let t1 = alloc.from("bool")?;
        let t2 = alloc.from("bool")?;
        assert_eq!(t1, t2);
        Ok(())
    }

    #[test]
    fn test_bool_neq() -> Result<()> {
        let (mut env, mut alloc) = setup_type_env()?;
        let t1 = alloc.from("int")?;
        let t2 = alloc.from("bool")?;
        assert_ne!(t1, t2);
        Ok(())
    }
}
