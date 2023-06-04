use anyhow::Result;
use ast::{ast::Program, into_ast::into_ast};
use interpreter_env::InterpreterEnv;
use log::debug;
use symbolic_expressions::parser::parse_str;

use crate::eval::Eval;

pub mod eval;
pub mod interpreter;
pub mod interpreter_env;
pub mod type_check;

fn main() -> Result<()> {
    let sexp = parse_str("(let a (: int) 1)")?;
    // "(+ 1 2)"
    // "(let a int 1)"
    // "(let x (app zero? 3))"
    // "(let a (app zero? 1))"
    // "(app (lam ((: x int)) x) 1)"
    let mut env: InterpreterEnv = Default::default();
    println!("{}", &env);
    let program = Program(into_ast(&mut env.alloc, &sexp)?);
    let ret = program.eval(&mut env)?;
    println!("eval: {:?} -> {}", program, &ret);
    Ok(())
}
