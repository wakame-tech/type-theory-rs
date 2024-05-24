use crate::environment::Environment;
use anyhow::Result;
use ast::ast::{Expr, External, FnDef, Parameter, Value};
use structural_typesystem::type_env::{arrow, TypeEnv};
use symbolic_expressions::parser::parse_str;

pub fn define_externals(type_env: &mut TypeEnv, env: &mut Environment) -> Result<()> {
    let int = || parse_str("int").unwrap();
    let bool = || parse_str("bool").unwrap();

    for (name, args, ret) in [
        ("+", vec![("a", int()), ("b", int())], int()),
        ("-", vec![("a", int()), ("b", int())], int()),
        ("not", vec![("a", bool())], bool()),
        ("&", vec![("a", bool()), ("b", bool())], bool()),
        ("|", vec![("a", bool()), ("b", bool())], bool()),
        ("==", vec![("a", int()), ("b", int())], bool()),
        ("!=", vec![("a", int()), ("b", int())], bool()),
        ("dbg", vec![("a", parse_str("a")?)], parse_str("a")?),
        ("id", vec![("a", parse_str("a")?)], parse_str("a")?),
        (
            "[]",
            vec![("r", parse_str("a")?), ("k", parse_str("b")?)],
            parse_str("([] a b)")?,
        ),
    ] {
        let ty = arrow(args.iter().map(|(_, arg)| arg).cloned().collect(), ret);
        log::debug!("{} : {}", name, ty);
        let ty = type_env.new_type(&ty)?;
        type_env.set_variable(name, ty);

        let def = Expr::FnDef(FnDef::new(
            args.into_iter()
                .map(|(name, typ)| Parameter::new(name.to_string(), Some(typ)))
                .collect(),
            Box::new(Expr::Literal(Value::External(External(name.to_string())))),
        ));
        env.variables.insert(name.to_string(), def);
    }
    Ok(())
}

pub fn eval_externals(env: Environment, name: &str) -> Result<(Expr, Environment)> {
    let res = match name {
        "dbg" => a_dbg(&env),
        "id" => a_id(&env),
        "+" => number_plus(&env),
        "-" => number_minus(&env),
        "not" => bool_not(&env),
        "&" => bool_and(&env),
        "|" => bool_or(&env),
        "==" => number_eq(&env),
        "!=" => number_neq(&env),
        "[]" => access(&env),
        _ => Err(anyhow::anyhow!("{} is not external", name)),
    }?;
    Ok((res, env))
}

fn a_dbg(env: &Environment) -> Result<Expr> {
    let a = env.get("a")?;
    println!("{}", a);
    Ok(a.clone())
}

fn a_id(env: &Environment) -> Result<Expr> {
    let a = env.get("a")?;
    Ok(a.clone())
}

fn number_plus(env: &Environment) -> Result<Expr> {
    let a = env.get("a")?.literal()?.number()?;
    let b = env.get("b")?.literal()?.number()?;
    Ok(Expr::Literal(Value::Number(a + b)))
}

fn number_minus(env: &Environment) -> Result<Expr> {
    let a = env.get("a")?.literal()?.number()?;
    let b = env.get("b")?.literal()?.number()?;
    Ok(Expr::Literal(Value::Number(a - b)))
}

fn number_eq(env: &Environment) -> Result<Expr> {
    let a = env.get("a")?.literal()?.number()?;
    let b = env.get("b")?.literal()?.number()?;
    Ok(Expr::Literal(Value::Bool(a == b)))
}

fn number_neq(env: &Environment) -> Result<Expr> {
    let a = env.get("a")?.literal()?.number()?;
    let b = env.get("b")?.literal()?.number()?;
    Ok(Expr::Literal(Value::Bool(a != b)))
}

fn bool_not(env: &Environment) -> Result<Expr> {
    let a = env.get("a")?.literal()?.boolean()?;
    Ok(Expr::Literal(Value::Bool(!a)))
}

fn bool_and(env: &Environment) -> Result<Expr> {
    let a = env.get("a")?.literal()?.boolean()?;
    let b = env.get("b")?.literal()?.boolean()?;
    Ok(Expr::Literal(Value::Bool(a && b)))
}

fn bool_or(env: &Environment) -> Result<Expr> {
    let a = env.get("a")?.literal()?.boolean()?;
    let b = env.get("b")?.literal()?.boolean()?;
    Ok(Expr::Literal(Value::Bool(a || b)))
}

fn access(env: &Environment) -> Result<Expr> {
    let r = env.get("r")?.literal()?;
    let r = r.record()?;
    let k = env.get("k")?.literal()?.atom()?;
    Ok(r.get(&k).unwrap().clone())
}
