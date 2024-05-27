use crate::{environment::Environment, eval::Eval};
use anyhow::Result;
use ast::ast::{Expr, FnApp, Value};
use structural_typesystem::type_env::TypeEnv;

pub fn eval_externals(
    t_env: &mut TypeEnv,
    env: Environment,
    name: &str,
    args: Vec<Expr>,
) -> Result<(Expr, Environment)> {
    let res = match name {
        "dbg" => a_dbg(&env, args),
        "to_string" => a_to_string(&env, args),
        "id" => a_id(&env, args),
        "+" => number_plus(&env, args),
        "-" => number_minus(&env, args),
        "%" => number_mod(&env, args),
        "not" => bool_not(&env, args),
        "&" => bool_and(&env, args),
        "|" => bool_or(&env, args),
        "==" => number_eq(&env, args),
        "!=" => number_neq(&env, args),
        "[]" => access(&env, args),
        "map" => map(t_env, &env, args),
        "filter" => filter(t_env, &env, args),
        "range" => range(&env, args),
        _ => Err(anyhow::anyhow!("{} is not external", name)),
    }?;
    Ok((res, env))
}

fn a_dbg(_env: &Environment, args: Vec<Expr>) -> Result<Expr> {
    let a = &args[0];
    println!("{}", a);
    Ok(a.clone())
}

fn a_to_string(_env: &Environment, args: Vec<Expr>) -> Result<Expr> {
    let v = &args[0];
    Ok(Expr::Literal(Value::String(format!("{}", v))))
}

fn a_id(_env: &Environment, args: Vec<Expr>) -> Result<Expr> {
    let a = &args[0];
    Ok(a.clone())
}

fn number_plus(_env: &Environment, args: Vec<Expr>) -> Result<Expr> {
    let a = &args[0].literal()?.number()?;
    let b = &args[1].literal()?.number()?;
    Ok(Expr::Literal(Value::Number(a + b)))
}

fn number_minus(_env: &Environment, args: Vec<Expr>) -> Result<Expr> {
    let a = &args[0].literal()?.number()?;
    let b = &args[1].literal()?.number()?;
    Ok(Expr::Literal(Value::Number(a - b)))
}

fn number_mod(_env: &Environment, args: Vec<Expr>) -> Result<Expr> {
    let a = &args[0].literal()?.number()?;
    let b = &args[1].literal()?.number()?;
    Ok(Expr::Literal(Value::Number(a % b)))
}

fn number_eq(_env: &Environment, args: Vec<Expr>) -> Result<Expr> {
    let a = &args[0].literal()?.number()?;
    let b = &args[1].literal()?.number()?;
    Ok(Expr::Literal(Value::Bool(a == b)))
}

fn number_neq(_env: &Environment, args: Vec<Expr>) -> Result<Expr> {
    let a = args[0].literal()?.number()?;
    let b = args[1].literal()?.number()?;
    Ok(Expr::Literal(Value::Bool(a != b)))
}

fn bool_not(_env: &Environment, args: Vec<Expr>) -> Result<Expr> {
    let a = args[0].literal()?.boolean()?;
    Ok(Expr::Literal(Value::Bool(!a)))
}

fn bool_and(_env: &Environment, args: Vec<Expr>) -> Result<Expr> {
    let a = args[0].literal()?.boolean()?;
    let b = args[1].literal()?.boolean()?;
    Ok(Expr::Literal(Value::Bool(a && b)))
}

fn bool_or(_env: &Environment, args: Vec<Expr>) -> Result<Expr> {
    let a = args[0].literal()?.boolean()?;
    let b = args[1].literal()?.boolean()?;
    Ok(Expr::Literal(Value::Bool(a || b)))
}

fn access(_env: &Environment, args: Vec<Expr>) -> Result<Expr> {
    let r = args[0].literal()?;
    let r = r.record()?;
    let k = args[1].literal()?.atom()?;
    Ok(r.get(&k).unwrap().clone())
}

fn map(t_env: &mut TypeEnv, env: &Environment, args: Vec<Expr>) -> Result<Expr> {
    log::debug!("map: {:?}", args);
    let f = &args[0];
    let v = args[1].literal()?;
    let v = v.list()?;
    let elements = v
        .iter()
        .map(|e| {
            let app = FnApp::new(f.clone(), vec![e.clone()]);
            log::debug!("{}", app);
            let (e, _) = app.eval(t_env, env.clone())?;
            Ok(e)
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Expr::Literal(Value::List(elements)))
}

fn filter(t_env: &mut TypeEnv, env: &Environment, args: Vec<Expr>) -> Result<Expr> {
    let v = args[1].literal()?;
    let v = v.list()?;
    let mut elements = vec![];
    for e in v {
        let ok = FnApp::new(args[0].clone(), vec![e.clone()])
            .eval(t_env, env.clone())
            .map(|t| t.0)?;
        if ok.literal()?.boolean()? {
            elements.push(e.clone());
        }
    }
    Ok(Expr::Literal(Value::List(elements)))
}

fn range(_env: &Environment, args: Vec<Expr>) -> Result<Expr> {
    let start = args[0].literal()?.number()?;
    let end = args[1].literal()?.number()?;
    Ok(Expr::Literal(Value::List(
        (start..end)
            .map(|i| Expr::Literal(Value::Number(i)))
            .collect(),
    )))
}
