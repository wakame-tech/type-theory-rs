use crate::{
    type_env::TypeEnv,
    types::{Id, Type},
};
use anyhow::{anyhow, Result};
use std::collections::HashSet;
use symbolic_expressions::parser::parse_str;

/// subtyping order for [TypeExpr]
///
/// ## examples
/// - `int` <= `any`
/// - `int -> int` <= `int -> int`
/// - `int -> int` <= `any`
/// - `{ a: int }` <= `{ a: any, b: int }`
pub fn is_subtype(env: &mut TypeEnv, a: Id, b: Id) -> Result<bool> {
    let any = env.get(&parse_str("any")?)?;
    let (a_ty, b_ty) = (env.alloc.from_id(a)?, env.alloc.from_id(b)?);

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
        ) if a_types.is_empty() && b_types.is_empty() => Ok(env.has_edge(a_id, b_id)),
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
            .map(|(ae, be)| is_subtype(env, *ae, *be))
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
                    is_subtype(env, *a_types.get(k).unwrap(), *b_types.get(k).unwrap())
                        .unwrap_or(false)
                }))
        }
        _ => Ok(false),
    }
}

#[cfg(test)]
mod test {
    use crate::{subtyping::is_subtype, type_env::TypeEnv};
    use anyhow::Result;
    use symbolic_expressions::parser::parse_str;

    #[test]
    fn test_type_cmp_1() -> Result<()> {
        let mut type_env = TypeEnv::default();
        let any = type_env.get(&parse_str("any")?)?;
        let int = type_env.get(&parse_str("int")?)?;

        assert!(is_subtype(&mut type_env, int, any)?, "int < any");
        Ok(())
    }

    #[test]
    fn test_type_cmp_2() -> Result<()> {
        let mut type_env = TypeEnv::default();
        let int_int = type_env.get(&parse_str("(-> int int)")?)?;
        let any_int = type_env.get(&parse_str("(-> any int)")?)?;
        assert!(
            is_subtype(&mut type_env, int_int, any_int)?,
            "int -> int <= int -> any"
        );
        Ok(())
    }

    #[test]
    fn test_type_cmp_3() -> Result<()> {
        let mut type_env = TypeEnv::default();
        let any = type_env.get(&parse_str("any")?)?;
        let int_int = type_env.get(&parse_str("(-> int int)")?)?;
        assert!(
            is_subtype(&mut type_env, int_int, any)?,
            "int -> int <= any"
        );
        Ok(())
    }

    #[test]
    fn test_type_cmp_record() -> Result<()> {
        let mut type_env = TypeEnv::default();
        let rec_a = type_env.get(&parse_str("(record (a int))")?)?;
        let rec_b = type_env.get(&parse_str("(record (a any) (b int))")?)?;
        assert!(
            is_subtype(&mut type_env, rec_a, rec_b)?,
            "{{ a: int }} <= {{ a: any, b: int }}"
        );
        Ok(())
    }
}
