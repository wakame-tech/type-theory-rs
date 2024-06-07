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
        if a == b {
            return Ok(true);
        }

        let any = self.get(&parse_str("any")?)?;
        let (a, b) = (type_eval(self, a)?, type_eval(self, b)?);
        let (a_ty, b_ty) = (self.alloc.get(a)?, self.alloc.get(b)?);
        let res = match (a_ty, b_ty) {
            // union types
            (_, Type::Union { types, .. }) => Ok(types
                .iter()
                .any(|t| self.is_subtype(a, *t).unwrap_or(false))),
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
            // ? vs any
            (_, Type::Primitive { id, .. }) if id == any => Ok(true),
            // atom literal types
            (Type::Primitive { name, .. }, _) if name.starts_with(":") => {
                let atom = self.get(&parse_str("atom")?)?;
                self.is_subtype(atom, b)
            }
            // int literal types
            (Type::Primitive { name, .. }, _) if name.parse::<i32>().is_ok() => {
                let int = self.get(&parse_str("int")?)?;
                self.is_subtype(int, b)
            }
            // str literal types
            (Type::Primitive { name, .. }, _) if name.starts_with("'") && name.ends_with("'") => {
                let str = self.get(&parse_str("str")?)?;
                self.is_subtype(str, b)
            }
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

    fn is_subtype(a: &str, b: &str) -> Result<bool> {
        let mut env = TypeEnv::default();
        let a = env.new_type(&parse_str(a)?)?;
        let b = env.new_type(&parse_str(b)?)?;
        env.is_subtype(a, b)
    }

    #[test]
    fn test_is_subtype_any() -> Result<()> {
        assert!(is_subtype("int", "any")?);
        assert!(is_subtype("((int) -> int)", "any")?);
        Ok(())
    }

    #[test]
    fn test_is_subtype_literal() -> Result<()> {
        assert!(is_subtype(":ok", "atom")?);
        assert!(is_subtype(":ok", "any")?);
        assert!(is_subtype(":ok", "(| :ok :err)")?);
        assert!(!is_subtype(":hoge", "(| :ok :err)")?);
        assert!(is_subtype("3", "int")?);
        assert!(is_subtype("3", "any")?);
        Ok(())
    }

    #[test]
    fn test_is_subtype_fn() -> Result<()> {
        assert!(is_subtype("((any) -> int)", "((int) -> any)")?);
        Ok(())
    }

    #[test]
    fn test_is_subtype_record() -> Result<()> {
        assert!(is_subtype(
            "(record (a : int) (b : int))",
            "(record (a : any) (b : int))",
        )?);
        Ok(())
    }

    #[test]
    fn test_is_subtype_union() -> Result<()> {
        assert!(is_subtype("int", "(| int any)")?);
        assert!(is_subtype("any", "(| int any)")?);
        assert!(is_subtype("3", "(| 1 2 3)")?);
        assert!(!is_subtype("3", "(| 0)")?);
        // assert!(is_subtype("(| 1)", "(| 1 2 3)")?);
        Ok(())
    }
}
