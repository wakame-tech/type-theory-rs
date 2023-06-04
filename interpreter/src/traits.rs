use anyhow::Result;
use ast::ast::Expr;
use structural_typesystem::types::Type;

use crate::interpreter_env::InterpreterEnv;

pub trait Eval {
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr>;
}

pub trait TypeCheck {
    fn type_check(&self, env: &InterpreterEnv) -> Result<Type>;
}
