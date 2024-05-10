use crate::{
    infer::{infer_type, infer_value_type},
    interpreter_env::InterpreterEnv,
    traits::TypeCheck,
};
use anyhow::Result;
use ast::ast::{Expr, FnApp, FnDef, Let, MacroApp, Program};
use std::collections::HashSet;
use structural_typesystem::{
    subtyping::is_subtype,
    type_env::TypeEnv,
    types::{Id, Type},
};
use symbolic_expressions::parser::parse_str;

fn ensure_subtype(env: &mut TypeEnv, a: Id, b: Id) -> Result<()> {
    if !is_subtype(env, a, b)? {
        return Err(anyhow::anyhow!(
            "{} is not subtype of {}",
            env.alloc.as_sexp(a, &mut Default::default())?,
            env.alloc.as_sexp(b, &mut Default::default())?
        ));
    }
    Ok(())
}

impl TypeCheck for FnDef {
    fn type_check(&self, env: &mut InterpreterEnv) -> Result<Id> {
        let param_typ = if let Some(typ) = &self.param.typ {
            env.type_env.new_type(typ)?
        } else {
            todo!()
        };
        env.new_var(
            &self.param.name,
            Expr::Variable(self.param.name.clone()),
            param_typ,
        );

        let body_typ = self.body.type_check(env)?;
        let f_typ = env.type_env.alloc.new_function(param_typ, body_typ);
        Ok(f_typ)
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
            infer_type(env, &self.value, &mut HashSet::new())?
        };
        env.new_var(&self.name, *self.value.clone(), let_ty);
        Ok(let_ty)
    }
}

impl TypeCheck for FnApp {
    /// f :: a -> b
    /// v :: a
    fn type_check(&self, env: &mut InterpreterEnv) -> Result<Id> {
        let f_ty = self.0.type_check(env)?;
        let Type::Operator { name, types, .. } = env.type_env.alloc.from_id(f_ty)? else {
            return Err(anyhow::anyhow!("{} is not appliable type", self.0));
        };
        anyhow::ensure!(name == "->");

        let (arg_ty, _ret_ty) = (types[0], types[1]);
        let param_ty = self.1.type_check(env)?;

        // if `arg_ty` is generic, skip subtype check
        if !env.type_env.alloc.is_generic(arg_ty)? {
            ensure_subtype(&mut env.type_env, param_ty, arg_ty)?;
        }

        // TODO: clone `TypeEnv` instead of `InterpreterEnv` when infer type of generic variables
        let ret_ty = infer_type(
            &mut env.clone(),
            &Expr::FnApp(self.clone()),
            &mut HashSet::new(),
        )?;
        Ok(ret_ty)
    }
}

impl TypeCheck for MacroApp {
    fn type_check(&self, env: &mut InterpreterEnv) -> Result<Id> {
        let ret_ty = match self.0.list()?[0].string()?.as_str() {
            "add!" => "int",
            "not!" => "bool",
            _ => panic!(),
        };
        env.type_env.get(&parse_str(ret_ty)?)
    }
}

impl TypeCheck for Expr {
    fn type_check(&self, env: &mut InterpreterEnv) -> Result<Id> {
        let ret = match self {
            Expr::Literal(value) => infer_value_type(env, value, &mut Default::default()),
            Expr::Variable(name) => Ok(env.get_variable(name)?.0),
            Expr::Let(lt) => lt.type_check(env),
            Expr::FnApp(app) => app.type_check(env),
            Expr::FnDef(fn_def) => fn_def.type_check(env),
            Expr::MacroApp(macro_app) => macro_app.type_check(env),
        }?;
        log::debug!(
            "type_check {} :: {}",
            self,
            env.type_env.alloc.as_sexp(ret, &mut Default::default())?
        );
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
            ("(let x : (record (a int)) (record (a 3)))", None),
            (
                "(let x : (record (a int) (b bool)) (record (b true)))",
                None,
            ),
            (
                "(let x : (record (a bool) (b int)) (record (b 1) (a 2)))",
                Some(
                    "(record (a int) (b int)) is not subtype of (record (a bool) (b int))"
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
