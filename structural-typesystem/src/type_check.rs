use crate::{
    infer::InferType,
    type_env::TypeEnv,
    type_eval::{ensure_subtype, type_eval},
    types::{Id, Type},
};
use anyhow::Result;
use ast::ast::{Case, Expr, FnApp, FnDef, Let, Program, TypeDef, Value};

use std::collections::{BTreeMap, HashSet};

pub trait TypeCheck {
    fn type_check(&self, env: &mut TypeEnv) -> Result<Id>;
}

impl TypeCheck for Value {
    fn type_check(&self, env: &mut TypeEnv) -> Result<Id> {
        match self {
            Value::Record(fields) => {
                let field_tys = fields
                    .iter()
                    .map(|(name, expr)| expr.type_check(env).map(|id| (name.to_string(), id)))
                    .collect::<Result<BTreeMap<_, _>>>()?;
                let id = env.alloc.issue_id();
                let record_ty = Type::record(id, field_tys);
                env.alloc.insert(record_ty);
                Ok(id)
            }
            Value::List(elems) => {
                let vec_ty = env.new_type_str("vec")?;
                // element type is the join of all element types
                let mut elem_ty = None;
                for elem in elems {
                    let ty = elem.type_check(env)?;
                    elem_ty = Some(match elem_ty {
                        Some(acc) => env.join(acc, ty)?,
                        None => ty,
                    });
                }
                let elem_ty = match elem_ty {
                    Some(ty) => ty,
                    None => env.new_type_str("a")?,
                };
                let container_ty = Type::container(vec_ty, vec![elem_ty]);
                let id = env.alloc.issue_id();
                env.alloc.insert(container_ty);
                Ok(id)
            }
            _ => self.infer_type(env, &Default::default()),
        }
    }
}

impl TypeCheck for FnDef {
    fn type_check(&self, env: &mut TypeEnv) -> Result<Id> {
        let arg_tys = self
            .args
            .iter()
            .map(|arg| {
                let arg_ty = if let Some(arg_ty) = &arg.typ {
                    env.new_type(arg_ty)?
                } else {
                    let id = env.alloc.issue_id();
                    env.alloc.insert(Type::variable(id, None));
                    id
                };
                env.set_variable(&arg.name, arg_ty);
                Ok(arg_ty)
            })
            .collect::<Result<Vec<_>>>()?;
        let ret_ty = self.body.type_check(env)?;
        let fn_ty = env.alloc.issue_id();
        env.alloc.insert(Type::function(fn_ty, arg_tys, ret_ty));
        Ok(fn_ty)
    }
}

impl TypeCheck for Let {
    fn type_check(&self, env: &mut TypeEnv) -> Result<Id> {
        log::debug!("let {} = {}", self.name, self.value);
        let use_decl_type = matches!(self.value.as_ref(), Expr::Literal(Value::External(_)));

        let let_ty = if self.typ.is_some() || use_decl_type {
            let decl_ty = self.typ.as_ref().unwrap();
            let decl_ty = env.new_type(decl_ty)?;
            let decl_ty = type_eval(env, decl_ty)?;
            if !use_decl_type {
                let value_ty = self.value.type_check(env)?;
                ensure_subtype(env, value_ty, decl_ty)?;
            }
            decl_ty
        } else {
            let infer_ty = self.value.infer_type(env, &HashSet::new())?;
            log::debug!("infer {} : {}", self.name, env.type_name(infer_ty)?);
            infer_ty
        };
        env.set_variable(&self.name, let_ty);
        log::debug!("{} : {}", self.name, env.type_name(let_ty)?);
        Ok(let_ty)
    }
}

impl TypeCheck for FnApp {
    /// f :: a -> b
    /// v :: a
    fn type_check(&self, env: &mut TypeEnv) -> Result<Id> {
        self.infer_type(env, &HashSet::new())?;
        let f_ty = self.0.type_check(env)?;
        let Type::Function { args, ret, .. } = env.alloc.get(f_ty)? else {
            return Err(anyhow::anyhow!("{} is not appliable type", self.0));
        };
        for (value, arg) in self.1.iter().zip(args.iter()) {
            let param_ty = value.type_check(env)?;
            // if `arg_ty` is generic, skip subtype check
            if let Type::Variable {
                upper_bound: Some(bound),
                ..
            } = env.alloc.get(*arg)?
            {
                ensure_subtype(env, *arg, bound)?;
            }
            if !env.alloc.is_generic(*arg)? {
                ensure_subtype(env, param_ty, *arg)?;
            }
        }
        Ok(ret)
    }
}

impl TypeCheck for TypeDef {
    fn type_check(&self, env: &mut TypeEnv) -> Result<Id> {
        let id = env.new_type(&self.typ)?;
        let id = type_eval(env, id)?;
        env.new_alias(&self.name, id);
        Ok(id)
    }
}

impl TypeCheck for Case {
    fn type_check(&self, env: &mut TypeEnv) -> Result<Id> {
        let bool_ty = env.new_type_str("bool")?;
        let mut ret_ty = None;
        for (pattern, body) in self.branches.iter() {
            let pattern_ty = pattern.type_check(env)?;
            if !env.is_subtype(pattern_ty, bool_ty)? {
                return Err(anyhow::anyhow!(
                    "pattern {} must be bool but {}",
                    pattern,
                    env.type_name(pattern_ty)?
                ));
            }
            // result type is the join of all branch types
            let body_ty = body.type_check(env)?;
            ret_ty = Some(match ret_ty {
                Some(acc) => env.join(acc, body_ty)?,
                None => body_ty,
            });
        }
        ret_ty.ok_or_else(|| anyhow::anyhow!("case must have at least one branch"))
    }
}

impl TypeCheck for Expr {
    fn type_check(&self, env: &mut TypeEnv) -> Result<Id> {
        let _span = tracing::debug_span!("", "{}", self).entered();
        let res = match self {
            Expr::Literal(value) => value.type_check(env),
            Expr::Variable(name) => env.get_variable(name),
            Expr::Let(lt) => lt.type_check(env),
            Expr::FnApp(app) => app.type_check(env),
            Expr::FnDef(fn_def) => fn_def.type_check(env),
            Expr::TypeDef(type_def) => type_def.type_check(env),
            Expr::Case(case) => case.type_check(env),
            Expr::Include(_) => Ok(env.new_type_str("str")?),
        }?;
        log::debug!(":{} #{}", env.type_name(res)?, res);
        Ok(res)
    }
}

impl TypeCheck for Program {
    fn type_check(&self, env: &mut TypeEnv) -> Result<Id> {
        let id = *self
            .0
            .iter()
            .map(|expr| expr.type_check(env))
            .collect::<Result<Vec<_>>>()?
            .last()
            .unwrap();
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use crate::{tests::setup, type_check::TypeCheck, type_env::TypeEnv};
    use anyhow::Result;
    use ast::into_ast::into_ast;
    use symbolic_expressions::parser::parse_str;

    #[test]
    fn r#let() -> Result<()> {
        setup();
        for (expected, error) in [
            ("(let x : (record (a : int)) (record (a : 3)))", None),
            (
                "(let x : (record (a : any) (b : bool)) (record (a : 1) (b : true)))",
                None,
            ),
            (
                "(let x : (record (a : bool) (b : int)) (record (b : 1) (a : 2)))",
                Some(
                    "(record (a : 2) (b : 1)) is not subtype of (record (a : bool) (b : int))"
                        .to_string(),
                ),
            ),
        ] {
            let ast = into_ast(&parse_str(expected)?)?;
            assert_eq!(
                ast.type_check(&mut Default::default())
                    .err()
                    .map(|e| e.to_string()),
                error
            );
        }
        Ok(())
    }

    #[test]
    fn bounded_type_variable() -> Result<()> {
        setup();
        for (expr, error) in [
            ("((fn (x : (a <: atom)) x) :a)", None),
            ("((fn (x : (a <: (| :ok :err))) x) :ok)", None),
            (
                "((fn (x : (a <: atom)) x) 1)",
                Some("1 is not subtype of atom".to_string()),
            ),
        ] {
            let ast = into_ast(&parse_str(expr)?)?;
            assert_eq!(
                ast.type_check(&mut Default::default())
                    .err()
                    .map(|e| e.to_string()),
                error
            );
        }
        Ok(())
    }

    fn type_of(expr: &str) -> Result<String> {
        setup();
        let ast = into_ast(&parse_str(expr)?)?;
        let mut env = TypeEnv::default();
        let id = ast.type_check(&mut env)?;
        Ok(env.type_name(id)?.to_string())
    }

    #[test]
    fn record_width() -> Result<()> {
        assert_eq!(
            type_of("(let x : (record (a : int)) (record (a : 1) (b : 2)))")?,
            "(record (a : int))"
        );
        assert!(type_of("(let x : (record (a : int) (b : int)) (record (a : 1)))").is_err());
        Ok(())
    }

    #[test]
    fn vec_join() -> Result<()> {
        assert_eq!(type_of("(vec 1 1)")?, "(vec 1)");
        assert_eq!(type_of("(vec true false)")?, "(vec (| true false))");
        assert_eq!(
            type_of("(let a : (vec bool) (vec true false))")?,
            "(vec bool)"
        );
        Ok(())
    }

    #[test]
    fn case_join() -> Result<()> {
        assert_eq!(type_of("(case (true => 1) (false => 2))")?, "(| 1 2)");
        assert_eq!(type_of("(case (true => 1) (false => 1))")?, "1");
        assert_eq!(
            type_of("(let x : int (case (true => 1) (false => 2)))")?,
            "int"
        );
        assert!(type_of("(case (1 => 1))").is_err());
        Ok(())
    }
}
