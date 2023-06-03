use anyhow::Result;
use ast::ast::Expr;

use crate::interpreter_env::InterpreterEnv;

pub trait Eval {
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr>;
}
