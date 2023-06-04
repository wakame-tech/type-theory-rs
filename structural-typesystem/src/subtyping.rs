use crate::{type_alloc::TypeAlloc, type_env::TypeEnv, types::Type};
use anyhow::Result;

/// subtyping order for [TypeExpr]
/// ## examples
/// any > int
/// any -> int > int -> int
pub fn is_subtype(env: &TypeEnv, alloc: &TypeAlloc, a: &Type, b: &Type) -> Result<bool> {
    dbg!(&a, &b);
    match (a, b) {
        (
            Type::Operator {
                id: a_id,
                name: a_name,
                types: a_types,
                ..
            },
            Type::Operator {
                id: b_id,
                name: b_name,
                types: b_types,
                ..
            },
        ) => {
            let ret = env.is_subtype(*a_id, *b_id)
                && a_types
                    .iter()
                    .zip(b_types.iter())
                    .map(|(ae, be)| {
                        is_subtype(env, alloc, &alloc.from_id(*ae)?, &alloc.from_id(*be)?)
                    })
                    .collect::<Result<Vec<_>>>()?
                    .iter()
                    .all(|e| *e);
            Ok(ret)
        }
        _ => todo!(),
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    use crate::{
        builtin_types::register_builtin_types, subtyping::is_subtype, type_alloc::TypeAlloc,
        type_env::TypeEnv,
    };

    #[test]
    fn test_type_cmp() -> Result<()> {
        let mut env = TypeEnv::new();
        let mut alloc = TypeAlloc::new();
        register_builtin_types(&mut env, &mut alloc)?;
        assert_eq!(
            is_subtype(
                &env,
                &alloc,
                &alloc.from_id(alloc.from("any")?)?,
                &alloc.from_id(alloc.from("bool")?)?,
            )?,
            true
        );
        Ok(())
    }
}
