use crate::traits::{Eval, TypeCheck};
use anyhow::Result;
use ast::{ast::Program, into_ast::into_ast};
use interpreter_env::InterpreterEnv;
use simple_logger::SimpleLogger;
use std::{env, fs::File, io::Read};
use symbolic_expressions::parser::parse_str;

pub mod infer;
pub mod interpreter;
pub mod interpreter_env;
pub mod scope;
pub mod traits;
pub mod type_check;

fn main() -> Result<()> {
    SimpleLogger::new()
        .without_timestamps()
        .with_level(log::LevelFilter::Debug)
        .init()?;

    let args = env::args().collect::<Vec<_>>();
    let ml_path = args.get(1).ok_or(anyhow::anyhow!("require ml_path"))?;
    let mut f = File::open(ml_path)?;
    let mut program = String::new();
    f.read_to_string(&mut program)?;
    let sexps = program
        .split('\n')
        .filter(|line| !line.is_empty())
        .map(|line| parse_str(line).map_err(|e| anyhow::anyhow!("{:?}", e)))
        .collect::<Result<Vec<_>>>()?;

    let program = sexps.iter().map(into_ast).collect::<Result<Vec<_>>>()?;
    let mut env = InterpreterEnv::default();
    let program = Program(program);
    program.type_check(&mut env)?;
    let ret = program.eval(&mut env)?;
    println!("{}", &ret);
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use simple_logger::SimpleLogger;
    use std::sync::Once;

    static INIT: Once = Once::new();

    pub fn setup() {
        INIT.call_once(|| {
            SimpleLogger::new()
                .without_timestamps()
                .with_level(log::LevelFilter::Debug)
                .init()
                .unwrap();
        });
    }
}
