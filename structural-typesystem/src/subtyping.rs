use crate::{
    type_env::TypeEnv,
    types::{Id, Type},
};
use anyhow::{anyhow, Result};
use std::collections::HashSet;
use symbolic_expressions::parser::parse_str;

impl TypeEnv {
    /// subtyping order for [TypeExpr]
    pub fn is_subtype(&mut self, a: Id, b: Id) -> Result<bool> {
        let any = self.get(&parse_str("any")?)?;
        let (a_ty, b_ty) = (self.alloc.from_id(a)?, self.alloc.from_id(b)?);
        log::debug!(
            "#{} = {} <: #{} = {}",
            a,
            self.type_name(a)?,
            b,
            self.type_name(b)?,
        );

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
            ) if a_types.is_empty() && b_types.is_empty() => Ok(self.has_edge(a_id, b_id)),
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
            ) if a_name == "->" && b_name == "->" => {
                let all_sub_type = a_types
                    .iter()
                    .zip(b_types.iter())
                    .map(|(ae, be)| self.is_subtype(*ae, *be))
                    .collect::<Result<Vec<_>>>()?
                    .iter()
                    .all(|e| *e);
                if !all_sub_type {
                    Err(anyhow!(
                        "not {} < {}",
                        self.alloc.as_sexp(a, &mut Default::default())?,
                        self.alloc.as_sexp(b, &mut Default::default())?
                    ))
                } else {
                    Ok(true)
                }
            }
            (Type::Variable { .. }, _) | (_, Type::Variable { .. }) => {
                Err(anyhow!("type variable can't compare"))
            }
            // record types
            (Type::Record { types: a_types, .. }, Type::Record { types: b_types, .. }) => {
                let a_keys = a_types.keys().collect::<HashSet<_>>();
                let b_keys = b_types.keys().collect::<HashSet<_>>();
                Ok(a_keys.is_subset(&b_keys)
                    && a_keys.into_iter().all(|k| {
                        self.is_subtype(*a_types.get(k).unwrap(), *b_types.get(k).unwrap())
                            .unwrap_or(false)
                    }))
            }
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::type_env::TypeEnv;
    use anyhow::Result;
    use symbolic_expressions::parser::parse_str;

    #[test]
    fn test_type_cmp_1() -> Result<()> {
        let mut env = TypeEnv::default();
        let any = env.get(&parse_str("any")?)?;
        let int = env.get(&parse_str("int")?)?;

        assert!(env.is_subtype(int, any)?, "int < any");
        Ok(())
    }

    #[test]
    fn test_type_cmp_2() -> Result<()> {
        let mut env = TypeEnv::default();
        let int_int = env.new_type(&parse_str("(-> int int)")?)?;
        let any_int = env.new_type(&parse_str("(-> any int)")?)?;
        assert!(
            env.is_subtype(int_int, any_int)?,
            "int -> int <= int -> any"
        );
        Ok(())
    }

    #[test]
    fn test_type_cmp_3() -> Result<()> {
        let mut env = TypeEnv::default();
        let any = env.new_type(&parse_str("any")?)?;
        let int_int = env.new_type(&parse_str("(-> int int)")?)?;
        assert!(env.is_subtype(int_int, any)?, "int -> int <= any");
        Ok(())
    }

    #[test]
    fn test_type_cmp_record() -> Result<()> {
        let mut env = TypeEnv::default();
        let rec_a = env.new_type(&parse_str("(record (a int))")?)?;
        let rec_b = env.new_type(&parse_str("(record (a any) (b int))")?)?;
        assert!(
            env.is_subtype(rec_a, rec_b)?,
            "{{ a: int }} <= {{ a: any, b: int }}"
        );
        Ok(())
    }
}
