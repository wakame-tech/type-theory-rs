use anyhow::Result;
use env::Env;

pub mod ast;
pub mod env;
pub mod interpreter;
pub mod into_ast;

fn main() -> Result<()> {
    let sexps = vec![
        // "(let (: a int) 1)"
        "(+ 1 2)",
    ];
    let mut env = Env::new();
    interpreter::interpret(&mut env, &sexps)?;
    Ok(())
}
