use crate::interpreter_env::Context;
use anyhow::Result;
use ast::ast::{Expr, FnDef, Parameter, Value};
use structural_typesystem::type_env::TypeEnv;
use symbolic_expressions::parser::parse_str;

pub fn main_context(type_env: &mut TypeEnv) -> Result<Context> {
    let mut context = Context::new("main");

    let not_id = type_env.new_type(&parse_str("(-> bool bool)")?)?;
    context.insert(
        "not",
        not_id,
        Expr::FnDef(FnDef::new(
            Parameter::new("v".to_string(), Some(parse_str("bool")?)),
            Box::new(Expr::Literal(Value::Nil)),
        )),
    );

    let succ_id = type_env.new_type(&parse_str("(-> int int)")?)?;
    context.insert("succ", succ_id, Expr::Variable("succ".to_string()));

    let id_id = type_env.new_type(&parse_str("(-> a a)")?)?;
    context.insert("id", id_id, Expr::Variable("id".to_string()));

    Ok(context)
}
