use crate::{
    interpreter_env::InterpreterEnv,
    traits::{InferType, TypeCheck},
};
use anyhow::Result;
use ast::ast::{Expr, FnApp, FnDef, Let, Program};
use std::collections::HashSet;
use structural_typesystem::{
    type_env::TypeEnv,
    types::{Id, Type},
};

fn ensure_subtype(env: &mut TypeEnv, a: Id, b: Id) -> Result<()> {
    if !env.is_subtype(a, b)? {
        return Err(anyhow::anyhow!(
            "{} is not subtype of {}",
            env.type_name(a)?,
            env.type_name(b)?
        ));
    }
    Ok(())
}

impl TypeCheck for FnDef {
    fn type_check(&self, env: &mut InterpreterEnv) -> Result<Id> {
        let arg_ty = if let Some(arg_ty) = &self.arg.typ {
            env.type_env.new_type(arg_ty)?
        } else {
            let id = env.type_env.alloc.issue_id();
            env.type_env.alloc.insert(Type::variable(id));
            id
        };
        env.type_env.set_variable(&self.arg.name, arg_ty);
        let ret_ty = self.body.type_check(env)?;
        let fn_ty = env.type_env.alloc.issue_id();
        env.type_env
            .alloc
            .insert(Type::function(fn_ty, arg_ty, ret_ty));
        Ok(fn_ty)
    }
}

impl TypeCheck for Let {
    fn type_check(&self, env: &mut InterpreterEnv) -> Result<Id> {
        let value_ty = self.value.type_check(env)?;
        let let_ty = if let Some(decl_ty) = &self.typ {
            let decl_ty = env.type_env.new_type(decl_ty)?;
            ensure_subtype(&mut env.type_env, value_ty, decl_ty)?;
            decl_ty
        } else {
            self.value.infer_type(env, &mut HashSet::new())?
        };
        env.type_env.set_variable(&self.name, let_ty);
        Ok(let_ty)
    }
}

impl TypeCheck for FnApp {
    /// f :: a -> b
    /// v :: a
    fn type_check(&self, env: &mut InterpreterEnv) -> Result<Id> {
        let f_ty = self.0.type_check(env)?;
        let Type::Function { arg, ret, .. } = env.type_env.alloc.get(f_ty)? else {
            return Err(anyhow::anyhow!("{} is not appliable type", self.0));
        };
        let param_ty = self.1.type_check(env)?;

        // if `arg_ty` is generic, skip subtype check
        if !env.type_env.alloc.is_generic(arg)? {
            ensure_subtype(&mut env.type_env, param_ty, arg)?;
        }
        Ok(ret)
    }
}

impl TypeCheck for Expr {
    fn type_check(&self, env: &mut InterpreterEnv) -> Result<Id> {
        log::debug!("type_check {}", self);
        let ret = match self {
            Expr::Literal(value) => value.infer_type(env, &mut Default::default()),
            Expr::Variable(name) => env.type_env.get_variable(name),
            Expr::Let(lt) => lt.type_check(env),
            Expr::FnApp(app) => app.type_check(env),
            Expr::FnDef(fn_def) => fn_def.type_check(env),
        }?;
        log::debug!("type_check {} :: {}", self, env.type_env.type_name(ret)?);
        Ok(ret)
    }
}

impl TypeCheck for Program {
    fn type_check(&self, env: &mut InterpreterEnv) -> Result<Id> {
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
    use crate::{tests::setup, traits::TypeCheck};
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
