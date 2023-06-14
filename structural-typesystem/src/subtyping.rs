use crate::{
    type_alloc::TypeAlloc,
    type_env::TypeEnv,
    types::{Id, Type},
};
use anyhow::{anyhow, Result};
use std::collections::HashSet;

/// subtyping order for [TypeExpr]
///
/// ## examples
/// - `int` <= `any`
/// - `int -> int` <= `int -> int`
/// - `int -> int` <= `any`
/// - `{ a: int }` <= `{ a: any, b: int }`
pub fn is_subtype(env: &TypeEnv, alloc: &TypeAlloc, a: Id, b: Id) -> Result<bool> {
    let any = alloc.from("any")?;

    let (a_ty, b_ty) = (alloc.from_id(a)?, alloc.from_id(b)?);
    println!("{} <=? {}", a_ty, b_ty);

    match (a_ty, b_ty) {
        // any vs ?
        (Type::Operator { id, .. }, _) if id == any => Ok(false),
        // ? vs any
        (_, Type::Operator { id, .. }) if id == any => Ok(true),
        // primitive types
        (
            Type::Operator {
                id: a_id,
                types: a_types,
                ..
            },
            Type::Operator {
                id: b_id,
                types: b_types,
                ..
            },
        ) if a_types.is_empty() && b_types.is_empty() => Ok(env.is_subtype(a_id, b_id)),
        // fn types
        (
            Type::Operator {
                types: a_types,
                name: a_name,
                ..
            },
            Type::Operator {
                types: b_types,
                name: b_name,
                ..
            },
        ) if a_name == "->" && b_name == "->" => Ok(a_types
            .iter()
            .zip(b_types.iter())
            .map(|(ae, be)| is_subtype(env, alloc, *ae, *be))
            .collect::<Result<Vec<_>>>()?
            .iter()
            .all(|e| *e)),
        (Type::Variable { .. }, _) | (_, Type::Variable { .. }) => {
            Err(anyhow!("type variable can't compare"))
        }
        // record types
        (Type::Record { types: a_types, .. }, Type::Record { types: b_types, .. }) => {
            let a_keys = a_types.keys().collect::<HashSet<_>>();
            let b_keys = b_types.keys().collect::<HashSet<_>>();
            Ok(a_keys.is_subset(&b_keys)
                && a_keys.into_iter().all(|k| {
                    is_subtype(
                        env,
                        alloc,
                        *a_types.get(k).unwrap(),
                        *b_types.get(k).unwrap(),
                    )
                    .unwrap_or(false)
                }))
        }
        (a_ty, b_ty) => Err(anyhow!("{} <= {} not supported", a_ty, b_ty)),
    }
}

#[cfg(test)]
mod test {
    use crate::{
        builtin_types::register_builtin_types, subtyping::is_subtype, type_alloc::TypeAlloc,
        type_env::TypeEnv,
    };
    use anyhow::Result;
    use std::collections::BTreeMap;

    #[test]
    fn test_type_cmp_1() -> Result<()> {
        let (mut env, mut alloc) = (TypeEnv::new(), TypeAlloc::new());
        register_builtin_types(&mut env, &mut alloc)?;

        let any = alloc.from("any")?;
        let int = alloc.from("int")?;

        assert!(is_subtype(&env, &alloc, int, any)?, "int < any");
        Ok(())
    }

    #[test]
    fn test_type_cmp_2() -> Result<()> {
        let (mut env, mut alloc) = (TypeEnv::new(), TypeAlloc::new());
        register_builtin_types(&mut env, &mut alloc)?;

        let any = alloc.from("any")?;
        let int = alloc.from("int")?;
        let int_int = alloc.new_function(int, int);
        let any_int = alloc.new_function(any, int);

        assert!(
            is_subtype(&env, &alloc, int_int, any_int)?,
            "int -> int <= int -> any"
        );
        Ok(())
    }

    #[test]
    fn test_type_cmp_3() -> Result<()> {
        let (mut env, mut alloc) = (TypeEnv::new(), TypeAlloc::new());
        register_builtin_types(&mut env, &mut alloc)?;

        let any = alloc.from("any")?;
        let int = alloc.from("int")?;
        let int_int = alloc.new_function(int, int);

        assert!(
            is_subtype(&env, &alloc, int_int, any)?,
            "int -> int <= any"
        );
        Ok(())
    }

    #[test]
    fn test_type_cmp_record() -> Result<()> {
        let (mut env, mut alloc) = (TypeEnv::new(), TypeAlloc::new());
        register_builtin_types(&mut env, &mut alloc)?;

        let int = alloc.from("int")?;
        let any = alloc.from("any")?;
        let rec_a = alloc.new_record(BTreeMap::from_iter(vec![("a".to_string(), int)]));
        let rec_b = alloc.new_record(BTreeMap::from_iter(vec![
            ("a".to_string(), any),
            ("b".to_string(), int),
        ]));
        assert!(
            is_subtype(&env, &alloc, rec_a, rec_b)?,
            "{{ a: int }} <= {{ a: any, b: int }}"
        );
        Ok(())
    }
}
