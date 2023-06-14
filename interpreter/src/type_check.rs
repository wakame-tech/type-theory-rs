use crate::{interpreter_env::InterpreterEnv, traits::TypeCheck};
use anyhow::{anyhow, Result};
use ast::ast::{Expr, FnApp, FnDef, Let, Program};
use structural_typesystem::{subtyping::is_subtype, types::Id};
use symbolic_expressions::Sexp;

impl TypeCheck for FnDef {
    fn type_check(&self, env: &mut InterpreterEnv) -> Result<Id> {
        let param_typ = env.type_env.alloc.from_sexp(&self.param.typ)?;
        let body_typ = self.body.type_check(env)?;
        let f_typ = env.type_env.alloc.new_function(param_typ, body_typ);
        Ok(f_typ)
    }
}

impl TypeCheck for Let {
    fn type_check(&self, env: &mut InterpreterEnv) -> Result<Id> {
        let value_typ = self.value.type_check(env)?;
        if let Some(typ) = &self.typ {
            let typ = env.type_env.alloc.from_sexp(typ)?;
            is_subtype(&mut env.type_env, value_typ, typ)?;
            Ok(value_typ)
        } else {
            Ok(value_typ)
        }
    }
}

impl TypeCheck for FnApp {
    /// f :: a -> b
    /// v :: a
    fn type_check(&self, env: &mut InterpreterEnv) -> Result<Id> {
        let _f_type = self.0.type_check(env)?;
        let v_type = self.1.type_check(env)?;
        Ok(v_type)
    }
}

impl TypeCheck for Expr {
    fn type_check(&self, env: &mut InterpreterEnv) -> Result<Id> {
        match self {
            Expr::Literal(value) => match &value.raw {
                Sexp::String(lit) if lit.parse::<usize>().is_ok() => env.type_env.get("int"),
                _ => panic!(),
            },
            Expr::Variable(name) => env
                .type_env
                .get_id(name)
                .ok_or(anyhow!("type of {} not found", name)),
            Expr::Let(lt) => lt.type_check(env),
            Expr::FnApp(app) => app.type_check(env),
            Expr::FnDef(fn_def) => fn_def.type_check(env),
        }
    }
}

impl TypeCheck for Program {
    fn type_check(&self, env: &mut InterpreterEnv) -> Result<Id> {
        self.0.type_check(env)
    }
}
