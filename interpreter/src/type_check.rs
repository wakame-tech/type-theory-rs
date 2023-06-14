use crate::{interpreter_env::InterpreterEnv, traits::TypeCheck};
use anyhow::{anyhow, Result};
use ast::ast::{Expr, FnApp, FnDef, Let, Program};
use structural_typesystem::types::Type;

impl TypeCheck for FnDef {
    fn type_check(&self, env: &InterpreterEnv) -> Result<Type> {
        self.body.type_check(env)
    }
}

impl TypeCheck for Let {
    fn type_check(&self, env: &InterpreterEnv) -> Result<Type> {
        env.alloc.from_id(self.type_id)
    }
}

impl TypeCheck for FnApp {
    fn type_check(&self, env: &InterpreterEnv) -> Result<Type> {
        let expected = env.alloc.from_id(self.0)?;
        let actual = self
            .1
            .iter()
            .map(|expr| expr.type_check(env))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .reduce(|acc, ty| Type::Operator {
                id: 0,
                name: "->".to_string(),
                types: vec![acc.id(), ty.id()],
            })
            .unwrap();
        println!("{} vs {}", expected, actual);
        todo!()
    }
}

impl TypeCheck for Expr {
    fn type_check(&self, env: &InterpreterEnv) -> Result<Type> {
        match self {
            Expr::Literal(value) => env.alloc.from_id(value.type_id),
            Expr::Variable(name) => env
                .type_env
                .get_id(name)
                .ok_or(anyhow!("type of {} not found", name))
                .and_then(|tid| env.alloc.from_id(tid)),
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
