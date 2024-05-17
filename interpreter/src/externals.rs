use crate::scope::Scope;
use anyhow::Result;
use ast::ast::{Expr, External, FnDef, Parameter, Value};
use structural_typesystem::type_env::{arrow, TypeEnv};
use symbolic_expressions::parser::parse_str;

pub fn define_externals(type_env: &mut TypeEnv, scope: &mut Scope) -> Result<()> {
    let int = || parse_str("int").unwrap();
    let bool = || parse_str("bool").unwrap();

    for (name, args, ret) in [
        ("+", vec![("a", int()), ("b", int())], int()),
        ("-", vec![("a", int()), ("b", int())], int()),
        ("not", vec![("a", bool())], bool()),
        ("==", vec![("a", int()), ("b", int())], bool()),
        ("!=", vec![("a", int()), ("b", int())], bool()),
        ("dbg", vec![("a", parse_str("a")?)], parse_str("a")?),
        ("id", vec![("a", parse_str("a")?)], parse_str("a")?),
    ] {
        let ty = arrow(args.iter().map(|(_, arg)| arg).cloned().collect(), ret);
        let ty = type_env.new_type(&ty)?;
        type_env.set_variable(name, ty);

        let def = Expr::FnDef(FnDef::new(
            args.into_iter()
                .map(|(name, typ)| Parameter::new(name.to_string(), Some(typ)))
                .collect(),
            Box::new(Expr::Literal(Value::External(External(name.to_string())))),
        ));
        scope.variables.insert(name.to_string(), def);
    }
    Ok(())
}

pub fn eval_externals(scope: &Scope, name: &str) -> Result<Expr> {
    match name {
        "dbg" => a_dbg(scope),
        "id" => a_id(scope),
        "+" => number_plus(scope),
        "-" => number_minus(scope),
        "not" => bool_not(scope),
        "==" => number_eq(scope),
        "!=" => number_neq(scope),
        _ => Err(anyhow::anyhow!("{} is not external", name)),
    }
}

fn a_dbg(scope: &Scope) -> Result<Expr> {
    let a = scope.get("a")?;
    println!("{}", a);
    Ok(a.clone())
}

fn a_id(scope: &Scope) -> Result<Expr> {
    let a = scope.get("a")?;
    Ok(a.clone())
}

fn number_plus(scope: &Scope) -> Result<Expr> {
    let a = scope.get("a")?.literal()?.number()?;
    let b = scope.get("b")?.literal()?.number()?;
    Ok(Expr::Literal(Value::Number(a + b)))
}

fn number_minus(scope: &Scope) -> Result<Expr> {
    let a = scope.get("a")?.literal()?.number()?;
    let b = scope.get("b")?.literal()?.number()?;
    Ok(Expr::Literal(Value::Number(a - b)))
}

fn number_eq(scope: &Scope) -> Result<Expr> {
    let a = scope.get("a")?.literal()?.number()?;
    let b = scope.get("b")?.literal()?.number()?;
    Ok(Expr::Literal(Value::Bool(a == b)))
}

fn number_neq(scope: &Scope) -> Result<Expr> {
    let a = scope.get("a")?.literal()?.number()?;
    let b = scope.get("b")?.literal()?.number()?;
    Ok(Expr::Literal(Value::Bool(a != b)))
}

fn bool_not(scope: &Scope) -> Result<Expr> {
    let a = scope.get("a")?.literal()?.boolean()?;
    Ok(Expr::Literal(Value::Bool(!a)))
}
