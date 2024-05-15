use crate::interpreter_env::InterpreterEnv;
use anyhow::Result;
use ast::ast::Expr;
use std::collections::HashSet;
use structural_typesystem::{type_env::TypeEnv, types::Id};

pub trait Eval {
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr>;
}

pub trait InferType {
    fn infer_type(&self, env: &mut TypeEnv, non_generic: &HashSet<Id>) -> Result<Id>;
}

pub trait TypeCheck {
    fn type_check(&self, env: &mut TypeEnv) -> Result<Id>;
}
