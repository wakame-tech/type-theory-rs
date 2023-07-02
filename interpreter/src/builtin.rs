use crate::interpreter_env::Context;
use anyhow::Result;
use ast::{
    ast::{Expr, FnDef, Value},
    into_ast::parse_parameter,
};
use structural_typesystem::type_env::TypeEnv;
use symbolic_expressions::{parser::parse_str, Sexp};

pub fn main_context(type_env: &mut TypeEnv) -> Result<Context> {
    let mut context = Context::new("main");
    let empty = Expr::Literal(Value::new(Sexp::Empty));

    let not_id = type_env.new_type(&parse_str("(-> bool bool)")?)?;
    context.insert(
        "not",
        not_id,
        Expr::FnDef(FnDef::new(
            parse_parameter(&parse_str("(v : bool)")?)?,
            Box::new(empty),
        )),
    );

    let succ_id = type_env.new_type(&parse_str("(-> int int)")?)?;
    context.insert("succ", succ_id, Expr::Variable("succ".to_string()));

    let id_id = type_env.new_type(&parse_str("(-> a a)")?)?;
    context.insert("id", id_id, Expr::Variable("id".to_string()));

    Ok(context)
}
