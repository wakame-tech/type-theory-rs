use crate::{
    type_env::TypeEnv,
    type_eval::type_eval,
    types::{Id, Type},
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
        Ok(a_keys == b_keys
            && a_keys.into_iter().all(|k| {
                self.is_subtype(*a.get(k).unwrap(), *b.get(k).unwrap())
                    .unwrap_or(false)
            }))
    }

    /// subtyping order for [TypeExpr]
    pub fn is_subtype(&mut self, a: Id, b: Id) -> Result<bool> {
        let any = self.get(&parse_str("any")?)?;
        let (a, b) = (type_eval(self, a)?, type_eval(self, b)?);
        let (a_ty, b_ty) = (self.alloc.get(a)?, self.alloc.get(b)?);
        let res = match (a_ty, b_ty) {
            // any vs ?
            (Type::Primitive { id, .. }, _) if id == any => Ok(false),
            // ? vs any
            (_, Type::Primitive { id, .. }) if id == any => Ok(true),
            // primitive types
            (Type::Primitive { id: a_id, .. }, Type::Primitive { id: b_id, .. }) => {
                Ok(self.has_edge(a_id, b_id))
            }
            // fn types
            (
                Type::Function {
                    args: a_args,
                    ret: a_ret,
                    ..
                },
                Type::Function {
                    args: b_args,
                    ret: b_ret,
                    ..
                },
            ) => Ok(self.is_subtype_vec(b_args, a_args)? && self.is_subtype(a_ret, b_ret)?),
            // record types
            (
                Type::Record {
                    fields: a_fields, ..
                },
                Type::Record {
                    fields: b_fields, ..
                },
            ) => self.is_subtype_map(a_fields, b_fields),
            (
                Type::Container {
                    elements: a_elements,
                    ..
                },
                Type::Container {
                    elements: b_elements,
                    ..
                },
            ) => self.is_subtype_vec(a_elements, b_elements),
            (Type::Variable { id: a_id, .. }, Type::Variable { id: b_id, .. }) => Ok(a_id == b_id),
            _ => Ok(false),
        };
        log::debug!(
            "check {} #{} <: {} #{} = {:?}",
            self.type_name(a)?,
            a,
            self.type_name(b)?,
            b,
            res
        );
        res
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

        assert!(env.is_subtype(int, any)?);
        Ok(())
    }

    #[test]
    fn test_type_cmp_2() -> Result<()> {
        let mut env = TypeEnv::default();
        let any_int = env.new_type(&parse_str("((any) -> int)")?)?;
        let int_any = env.new_type(&parse_str("((int) -> any)")?)?;
        assert!(env.is_subtype(any_int, int_any)?);
        Ok(())
    }

    #[test]
    fn test_type_cmp_3() -> Result<()> {
        let mut env = TypeEnv::default();
        let any = env.new_type(&parse_str("any")?)?;
        let int_int = env.new_type(&parse_str("((int) -> int)")?)?;
        assert!(env.is_subtype(int_int, any)?);
        Ok(())
    }

    #[test]
    fn test_type_cmp_record() -> Result<()> {
        let mut env = TypeEnv::default();
        let rec_a = env.new_type(&parse_str("(record (a : int) (b : int))")?)?;
        let rec_b = env.new_type(&parse_str("(record (a : any) (b : int))")?)?;
        assert!(env.is_subtype(rec_a, rec_b)?);
        Ok(())
    }
}
