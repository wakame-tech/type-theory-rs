use crate::{environment::Environment, eval::Eval};
use anyhow::Result;
use ast::{
    ast::{Expr, Program},
    into_ast::into_ast,
};
use std::{env, fs::File, io::Read, path::PathBuf};
use structural_typesystem::{type_check::TypeCheck, type_env::TypeEnv};
use symbolic_expressions::parser::parse_str;

pub mod environment;
pub mod eval;
pub mod externals;

fn include(path: &PathBuf) -> Result<Program> {
    let mut f = File::open(path)?;
    let mut program = String::new();
    f.read_to_string(&mut program)?;
    parse(&program)
}

fn parse(program: &str) -> Result<Program> {
    let program = program
        .split('\n')
        .filter(|line| !line.starts_with(';'))
        .collect::<Vec<_>>()
        .join(" ");
    let program = parse_str(&format!("({})", program))?
        .list()?
        .iter()
        .map(into_ast)
        .collect::<Result<Vec<_>>>()?;
    Ok(Program(program))
}

pub fn setup_logger() {
    tracing_subscriber::fmt()
        .without_time()
        .with_max_level(tracing::Level::DEBUG)
        .with_line_number(true)
        .init();
}

fn main() -> Result<()> {
    let args = env::args().collect::<Vec<_>>();
    let ml_path = args.get(1).ok_or(anyhow::anyhow!("require ml_path"))?;
    let program = include(&PathBuf::from(ml_path))?;

    let mut type_env = TypeEnv::default();
    let mut env = Environment::new(None);

    setup_logger();
    for e in program.0.iter() {
        if let Expr::Include(path) = e {
            let module = include(&PathBuf::from(path))?;
            module.type_check(&mut type_env)?;
            (_, env) = module.eval(&mut type_env, env)?;
        }
    }
    program.type_check(&mut type_env)?;
    let (ret, _) = program.eval(&mut type_env, env)?;
    log::debug!("{}", &ret);
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use std::sync::Once;

    static INIT: Once = Once::new();

    pub fn setup() {
        INIT.call_once(|| {
            tracing_subscriber::fmt()
                .with_test_writer()
                .without_time()
                .with_max_level(tracing::Level::DEBUG)
                .with_line_number(true)
                .init();
        });
    }
}
