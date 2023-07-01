use crate::interpreter_env::Context;
use anyhow::Result;

use structural_typesystem::type_env::TypeEnv;


pub fn main_context(_type_env: &mut TypeEnv) -> Result<Context> {
    let context = Context::new("main");
    // id
    // let a = type_env.get("*")?;
    // let a_a = type_env.alloc.new_function(a, a);
    // type_env.add("v", a);
    // type_env.add("id", a_a);

    // let id_fn = FnDef::new(
    //     Parameter::new("v".to_string(), Sexp::String("a".to_string())),
    //     Box::new(Expr::Variable("id".to_string())),
    // );
    // context
    //     .variables
    //     .insert("id".to_string(), Expr::FnDef(id_fn));
    Ok(context)
}
