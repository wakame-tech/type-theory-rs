use crate::interpreter_env::Context;
use anyhow::Result;
use ast::ast::{Expr, FnDef, Parameter};
use structural_typesystem::type_env::TypeEnv;
use symbolic_expressions::Sexp;

pub fn main_context(type_env: &mut TypeEnv) -> Result<Context> {
    let mut context = Context::new();
    let int_int_int = type_env.get("(-> (-> int int) int)")?;
    type_env.add("+", int_int_int);
    let int = type_env.get("int")?;

    type_env.add("a", int);
    type_env.add("b", int);
    context
        .variables
        .insert("+".to_string(), Expr::Variable("+".to_string()));

    let a = type_env.get("*")?;
    let a_a = type_env.alloc.new_function(a, a);
    type_env.add("v", a);
    type_env.add("id", a_a);

    let id_fn = FnDef::new(
        Parameter::new("v".to_string(), Sexp::String("a".to_string())),
        Box::new(Expr::Variable("id".to_string())),
    );
    context
        .variables
        .insert("id".to_string(), Expr::FnDef(id_fn));
    Ok(context)
}
