use crate::{interpreter_env::InterpreterEnv, traits::TypeCheck};
use anyhow::{anyhow, Result};
use ast::ast::{Expr, FnApp, FnDef, Let, Program};
use structural_typesystem::types::Type;

impl TypeCheck for FnDef {
    fn type_check(&self, env: &InterpreterEnv) -> Result<Type> {
        todo!()
    }
}

impl TypeCheck for Let {
    fn type_check(&self, env: &InterpreterEnv) -> Result<Type> {
        env.alloc.from_id(self.type_id)
    }
}

impl TypeCheck for FnApp {
    fn type_check(&self, env: &InterpreterEnv) -> Result<Type> {
        todo!()
    }
}

impl TypeCheck for Expr {
    fn type_check(&self, env: &InterpreterEnv) -> Result<Type> {
        match self {
            Expr::Literal(value) => env.alloc.from_id(value.type_id),
            Expr::Variable(name) => env
                .variables
                .get(name)
                .ok_or(anyhow!("var {} not found", name))
                .and_then(|v| env.alloc.from_id(v.type_id())),
            Expr::Let(lt) => lt.type_check(env),
            Expr::FnApp(app) => app.type_check(env),
            Expr::FnDef(fn_def) => fn_def.type_check(env),
        }
    }
}

impl TypeCheck for Program {
    fn type_check(&self, env: &InterpreterEnv) -> Result<Type> {
        self.0.type_check(env)
    }
}
