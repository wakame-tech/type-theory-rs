use crate::{environment::Environment, eval::Eval, externals::define_externals};
use anyhow::Result;
use ast::{ast::Program, into_ast::into_ast};
use std::{env, fs::File, io::Read, path::PathBuf};
use structural_typesystem::{type_check::TypeCheck, type_env::TypeEnv};
use symbolic_expressions::parser::parse_str;

pub mod environment;
pub mod eval;
pub mod externals;

fn import(path: &PathBuf) -> Result<Program> {
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
    setup_logger();
    let args = env::args().collect::<Vec<_>>();
    let ml_path = args.get(1).ok_or(anyhow::anyhow!("require ml_path"))?;
    let prelude = import(&PathBuf::from("codes/prelude.sexp"))?;
    let program = import(&PathBuf::from(ml_path))?;

    let mut type_env = TypeEnv::default();
    let mut env = Environment::new(None);
    define_externals(&mut type_env, &mut env).unwrap();

    prelude.type_check(&mut type_env)?;
    program.type_check(&mut type_env)?;
    log::debug!("eval prelude");
    let (_, env) = prelude.eval(&mut type_env, env)?;
    let (ret, _) = program.eval(&mut type_env, env)?;
    log::debug!("{}", &ret);
    Ok(())
}
