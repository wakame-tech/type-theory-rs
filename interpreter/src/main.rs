use crate::{environment::Environment, eval::Eval, externals::define_externals};
use anyhow::Result;
use ast::{ast::Program, into_ast::into_ast};
use simple_logger::SimpleLogger;
use std::{env, fs::File, io::Read};
use structural_typesystem::{type_check::TypeCheck, type_env::TypeEnv};
use symbolic_expressions::parser::parse_str;

pub mod environment;
pub mod eval;
pub mod externals;

fn parse(program: &str) -> Result<Program> {
    let program = program
        .split("\n")
        .into_iter()
        .filter(|line| !line.starts_with(";"))
        .collect::<Vec<_>>()
        .join(" ");
    let program = parse_str(&format!("({})", program))?
        .list()?
        .iter()
        .map(into_ast)
        .collect::<Result<Vec<_>>>()?;
    Ok(Program(program))
}

fn main() -> Result<()> {
    SimpleLogger::new()
        .without_timestamps()
        .with_level(log::LevelFilter::Debug)
        .init()?;

    let args = env::args().collect::<Vec<_>>();
    let ml_path = args.get(1).ok_or(anyhow::anyhow!("require ml_path"))?;
    log::debug!("ml_path: {}", ml_path);
    let mut f = File::open(ml_path)?;
    let mut program = String::new();
    f.read_to_string(&mut program)?;

    let program = parse(&program)?;

    let mut type_env = TypeEnv::default();
    let mut env = Environment::new(None);
    define_externals(&mut type_env, &mut env).unwrap();

    program.type_check(&mut type_env)?;
    let (ret, _) = program.eval(&mut type_env, env)?;
    println!("{}", &ret);
    Ok(())
}
