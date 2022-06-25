use anyhow::Result;
use interpreter_env::InterpreterEnv;

pub mod ast;
pub mod interpreter;
pub mod interpreter_env;
pub mod into_ast;

fn main() -> Result<()> {
    let sexps = vec![
        // "(let a int 1)"
        // "(+ 1 2)",
        // "(let a int 1)",
        "(app (lam ((: x int)) x) 1)",
    ];
    let mut env = InterpreterEnv::new();
    interpreter::interpret(&mut env, &sexps)?;
    println!("{}", &env);
    Ok(())
}
