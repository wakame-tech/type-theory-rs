use crate::interpreter_env::InterpreterEnv;
use anyhow::Result;
use ast::ast::Expr;
use structural_typesystem::types::Id;

pub trait Eval {
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr>;
}

pub trait TypeCheck {
    fn type_check(&self, env: &mut InterpreterEnv) -> Result<Id>;
}
