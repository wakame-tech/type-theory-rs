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
                let elem_tys = elems
                    .iter()
                    .map(|elem| elem.type_check(env))
                    .collect::<Result<Vec<_>>>()?;
                if elem_tys.iter().collect::<HashSet<_>>().len() != 1 {
                    return Err(anyhow::anyhow!(
                        "list elements must have same type: [{}]",
                        elem_tys
                            .iter()
                            .map(|id| env.type_name(*id).map(|t| t.to_string()))
                            .collect::<Result<Vec<_>>>()?
                            .join(", ")
                    ));
                }
                let elem_ty = elem_tys[0];
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
                    env.alloc.insert(Type::variable(id));
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
        let body_tys = self
            .branches
            .iter()
            .map(|(pattern, body)| {
                let pattern_ty = pattern.type_check(env)?;
                if pattern_ty != env.new_type_str("bool")? {
                    return Err(anyhow::anyhow!(
                        "pattern {} must be bool but {}",
                        pattern,
                        env.type_name(pattern_ty)?
                    ));
                }
                let body_ty = body.type_check(env)?;
                Ok(body_ty)
            })
            .collect::<Result<Vec<_>>>()?;
        if body_tys.iter().collect::<HashSet<_>>().len() != 1 {
            return Err(anyhow::anyhow!(
                "case branches must have same type: [{}]",
                body_tys
                    .iter()
                    .map(|id| env.type_name(*id).map(|t| t.to_string()))
                    .collect::<Result<Vec<_>>>()?
                    .join(", ")
            ));
        }
        let ret_ty = body_tys[0];
        Ok(ret_ty)
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
    use crate::{tests::setup, type_check::TypeCheck};
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
                    "(record (a : int) (b : int)) is not subtype of (record (a : bool) (b : int))"
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
}
