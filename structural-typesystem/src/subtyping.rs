use crate::{
    type_env::TypeEnv,
    types::{Id, Type, RECORD_TYPE_KEYWORD},
};
use anyhow::Result;
use std::collections::{BTreeMap, HashSet};
use symbolic_expressions::parser::parse_str;

impl TypeEnv {
    fn is_subtype_vec(&mut self, a: Vec<Id>, b: Vec<Id>) -> Result<bool> {
        Ok(a.len() == b.len()
            && a.iter()
                .zip(b.iter())
                .map(|(ae, be)| self.is_subtype(*ae, *be))
                .collect::<Result<Vec<_>>>()?
                .iter()
                .all(|e| *e))
    }

    fn is_subtype_map(&mut self, a: BTreeMap<String, Id>, b: BTreeMap<String, Id>) -> Result<bool> {
        let a_keys = a.keys().collect::<HashSet<_>>();
        let b_keys = b.keys().collect::<HashSet<_>>();
        Ok(a_keys.is_subset(&b_keys)
            && a_keys.into_iter().all(|k| {
                self.is_subtype(*a.get(k).unwrap(), *b.get(k).unwrap())
                    .unwrap_or(false)
            }))
    }

    /// subtyping order for [TypeExpr]
    pub fn is_subtype(&mut self, a: Id, b: Id) -> Result<bool> {
        let any = self.get(&parse_str("any")?)?;
        let (a_ty, b_ty) = (self.alloc.get(a)?, self.alloc.get(b)?);
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
                    op: a_name,
                    ..
                },
                Type::Operator {
                    types: b_types,
                    op: b_name,
                    ..
                },
            ) if a_name == b_name => self.is_subtype_vec(
                a_types.values().cloned().collect(),
                b_types.values().cloned().collect(),
            ),
            // record types
            (
                Type::Operator {
                    op: a_name,
                    types: a_types,
                    ..
                },
                Type::Operator {
                    op: b_name,
                    types: b_types,
                    ..
                },
            ) if a_name == RECORD_TYPE_KEYWORD && b_name == RECORD_TYPE_KEYWORD => {
                let a_types = a_types
                    .iter()
                    .map(|(k, v)| (k.as_ref().unwrap().clone(), *v))
                    .collect::<BTreeMap<_, _>>();
                let b_types = b_types
                    .iter()
                    .map(|(k, v)| (k.as_ref().unwrap().clone(), *v))
                    .collect::<BTreeMap<_, _>>();
                self.is_subtype_map(a_types, b_types)
            }
            (Type::Variable { .. }, _) | (_, Type::Variable { .. }) => Ok(true),
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
