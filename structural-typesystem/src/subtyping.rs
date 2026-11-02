use crate::{
    type_env::TypeEnv,
    type_eval::type_eval,
    types::{Id, Type, UNION_TYPE_KEYWORD},
};
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use symbolic_expressions::{parser::parse_str, Sexp};

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

    /// width and depth subtyping: `a` has every label of `b` with a subtype field
    fn is_subtype_map(&mut self, a: BTreeMap<String, Id>, b: BTreeMap<String, Id>) -> Result<bool> {
        for (label, b_field) in b.iter() {
            let Some(a_field) = a.get(label) else {
                return Ok(false);
            };
            if !self.is_subtype(*a_field, *b_field)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// least upper bound of `a` and `b`: the larger one if comparable, otherwise their union
    pub fn join(&mut self, a: Id, b: Id) -> Result<Id> {
        if self.is_subtype(a, b)? {
            return Ok(b);
        }
        if self.is_subtype(b, a)? {
            return Ok(a);
        }
        let mut members = vec![];
        for t in [a, b] {
            let t = type_eval(self, t)?;
            match self.alloc.get(t)? {
                Type::Union { types, .. } => members.extend(types),
                _ => members.push(t),
            }
        }
        // drop members subsumed by another member
        let mut types = BTreeSet::new();
        for &m in members.iter() {
            let mut subsumed = false;
            for &o in members.iter().chain(types.clone().iter()) {
                if o != m && self.is_subtype(m, o)? && !self.is_subtype(o, m)? {
                    subsumed = true;
                    break;
                }
            }
            for &k in types.clone().iter() {
                if self.is_subtype(m, k)? && self.is_subtype(k, m)? {
                    subsumed = true;
                }
            }
            if !subsumed {
                types.insert(m);
            }
        }
        if types.len() == 1 {
            return Ok(*types.first().unwrap());
        }
        if types
            .iter()
            .any(|t| self.alloc.is_generic(*t).unwrap_or(true))
        {
            // type variables cannot round-trip through their names
            let id = self.alloc.issue_id();
            self.alloc.insert(Type::Union { id, types });
            return Ok(id);
        }
        let union = types
            .iter()
            .map(|t| self.type_name(*t))
            .collect::<Result<Vec<_>>>()?;
        let union = [Sexp::String(UNION_TYPE_KEYWORD.to_string())]
            .into_iter()
            .chain(union)
            .collect::<Vec<_>>();
        self.new_type(&Sexp::List(union))
    }

    /// subtyping order for [TypeExpr]
    pub fn is_subtype(&mut self, a: Id, b: Id) -> Result<bool> {
        if a == b {
            return Ok(true);
        }

        let any = self.get(&parse_str("any")?)?;
        let (a, b) = (type_eval(self, a)?, type_eval(self, b)?);
        if a == b {
            return Ok(true);
        }
        let (a_ty, b_ty) = (self.alloc.get(a)?, self.alloc.get(b)?);
        let res = match (a_ty, b_ty) {
            // left union types: every member must be a subtype
            (Type::Union { types, .. }, _) => Ok(types
                .iter()
                .all(|t| self.is_subtype(*t, b).unwrap_or(false))),
            // bounded type variables: the upper bound must be a subtype
            (
                Type::Variable {
                    upper_bound: Some(bound),
                    ..
                },
                _,
            ) => self.is_subtype(bound, b),
            // right union types: some member must be a supertype
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
            (Type::Primitive { name, .. }, _) if name.starts_with(':') => {
                let atom = self.get(&parse_str("atom")?)?;
                self.is_subtype(atom, b)
            }
            // int literal types
            (Type::Primitive { name, .. }, _) if name.parse::<i32>().is_ok() => {
                let int = self.get(&parse_str("int")?)?;
                self.is_subtype(int, b)
            }
            // str literal types
            (Type::Primitive { name, .. }, _) if name.starts_with('\'') && name.ends_with('\'') => {
                let str = self.get(&parse_str("str")?)?;
                self.is_subtype(str, b)
            }
            // bool literal types
            (Type::Primitive { name, .. }, _) if name == "true" || name == "false" => {
                let bool = self.get(&parse_str("bool")?)?;
                self.is_subtype(bool, b)
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
        log::debug!("{}", env.type_name(a)?);
        log::debug!("{}", env.type_name(b)?);
        env.is_subtype(a, b)
    }

    fn join(a: &str, b: &str) -> Result<String> {
        let mut env = TypeEnv::default();
        let a = env.new_type(&parse_str(a)?)?;
        let b = env.new_type(&parse_str(b)?)?;
        let j = env.join(a, b)?;
        Ok(env.type_name(j)?.to_string())
    }

    #[test]
    fn test_join() -> Result<()> {
        assert_eq!(join("1", "int")?, "int");
        assert_eq!(join("int", "1")?, "int");
        assert_eq!(join("1", "2")?, "(| 1 2)");
        assert_eq!(join("(| 1 :a)", "int")?, "(| int :a)");
        assert_eq!(join("(| 1 2)", "(| 2 3)")?, "(| 1 2 3)");
        assert_eq!(join("int", "any")?, "any");
        Ok(())
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
    fn test_is_subtype_record_width() -> Result<()> {
        assert!(is_subtype(
            "(record (a : int) (b : int))",
            "(record (a : int))"
        )?);
        assert!(is_subtype(
            "(record (a : 1) (b : true))",
            "(record (a : int))"
        )?);
        assert!(is_subtype("(record (a : int))", "(record)")?);
        assert!(!is_subtype(
            "(record (a : int))",
            "(record (a : int) (b : int))"
        )?);
        assert!(!is_subtype("(record (a : int))", "(record (b : int))")?);
        assert!(!is_subtype(
            "(record (a : bool) (b : int))",
            "(record (a : int))"
        )?);
        Ok(())
    }

    #[test]
    fn test_is_subtype_refl_after_eval() -> Result<()> {
        assert!(is_subtype("([] (record (x : int)) :x)", "int")?);
        assert!(is_subtype("int", "([] (record (x : int)) :x)")?);
        assert!(is_subtype("(| int)", "int")?);
        Ok(())
    }

    #[test]
    fn test_is_subtype_union() -> Result<()> {
        assert!(is_subtype("int", "(| int any)")?);
        assert!(is_subtype("any", "(| int any)")?);
        assert!(is_subtype("3", "(| 1 2 3)")?);
        assert!(!is_subtype("3", "(| 0)")?);
        assert!(is_subtype("(| 1 3)", "(| 1 2 3)")?);
        assert!(!is_subtype("(| 1 3)", "(| 1 2)")?);
        assert!(is_subtype("(| 1)", "(| 1 2 3)")?);
        assert!(is_subtype("(| 1 3)", "(| (| 1 2) 3)")?);
        assert!(is_subtype("(| 1 2 3)", "(| (| 1 2) (| 3))")?);
        assert!(is_subtype("(| int bool)", "(| int bool any)")?);
        assert!(is_subtype("(| str)", "(| int bool any)")?);
        Ok(())
    }

    #[test]
    fn test_is_subtype_bounded_variable() -> Result<()> {
        assert!(is_subtype("(a <: atom)", "atom")?);
        assert!(is_subtype("(a <: atom)", "any")?);
        assert!(is_subtype("(a <: :ok)", "(| :ok :err)")?);
        assert!(is_subtype("(a <: (| 1 2))", "int")?);
        assert!(is_subtype("(a <: (| 1 2))", "(| 1 2 3)")?);
        assert!(!is_subtype("(a <: atom)", "int")?);
        assert!(!is_subtype("a", "atom")?);
        Ok(())
    }

    #[test]
    fn test_is_subtype_variable() -> Result<()> {
        assert!(is_subtype("a", "a")?);
        assert!(!is_subtype("a", "b")?);
        assert!(is_subtype("a", "any")?);
        Ok(())
    }

    #[test]
    fn test_is_subtype_union_forall_exists() -> Result<()> {
        assert!(is_subtype("(| 1 :ok)", "(| int atom)")?);
        assert!(is_subtype("(| 1 2)", "int")?);
        assert!(is_subtype("(| 1 :ok)", "any")?);
        assert!(!is_subtype("(| 1 :ok)", "int")?);
        assert!(!is_subtype("(| 1 'a')", "(| int atom)")?);
        // transitivity: (| 1 :ok) <: (| 1 atom) <: (| int atom)
        assert!(is_subtype("(| 1 :ok)", "(| 1 atom)")?);
        assert!(is_subtype("(| 1 atom)", "(| int atom)")?);
        Ok(())
    }
}
