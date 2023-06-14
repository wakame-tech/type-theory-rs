use crate::traits::{Eval, TypeCheck};
use anyhow::Result;
use ast::{ast::Program, into_ast::into_ast};
use interpreter_env::InterpreterEnv;
use symbolic_expressions::parser::parse_str;

pub mod builtin;
pub mod interpreter;
pub mod interpreter_env;
pub mod traits;
pub mod type_check;

fn main() -> Result<()> {
    // let sexp = parse_str("(let a (: int) 1)")?;
    let sexp = parse_str("((+ 1) 2)")?;
    // "(+ 1 2)"
    // "(let a int 1)"
    // "(let x (app zero? 3))"
    // "(let a (app zero? 1))"
    // "(app (lam ((: x int)) x) 1)"
    let mut env = InterpreterEnv::default();
    println!("{}", &env);
    let program = Program(into_ast(&sexp)?);
    program.type_check(&mut env)?;
    let ret = program.eval(&mut env)?;
    println!("eval: {:?} -> {}", program, &ret);
    Ok(())
}
