use anyhow::Result;
use interpreter_env::InterpreterEnv;
use log::debug;

pub mod eval;
pub mod interpreter;
pub mod interpreter_env;
pub mod type_check;

fn main() -> Result<()> {
    let sexps = vec![
        "(let a (: int) 1)", // "(+ 1 2)",
                             // "(let a int 1)",
                             // "(let x (app zero? 3))",
                             // "(let a (app zero? 1))",
                             // "(app (lam ((: x int)) x) 1)",
    ];
    let mut env: InterpreterEnv = Default::default();
    println!("{}", &env);
    interpreter::interpret(&mut env, &sexps)?;
    Ok(())
}
