use structural_typesystem::types::Type;
use symbolic_expressions::Sexp;

use anyhow::Result;

use crate::{
    ast::{Expr, Parameter},
    interpreter_env::InterpreterEnv,
};

fn is_number(s: &str) -> bool {
    s.chars().all(|c| c.is_numeric())
}

pub fn into_variable(env: &mut InterpreterEnv, sexp: &Sexp) -> Result<Parameter> {
    match sexp {
        // with type annotation
        Sexp::List(list) => {
            if list[0].string()? != ":" {
                Err(anyhow::anyhow!("type annotation expected"))
            } else {
                let (name, annotation) = (list[1].string()?, list[2].string()?);
                let typ = Type::from(&env.alloc, &annotation)?;
                Ok(Parameter::new(name.clone(), typ))
            }
        }
        _ => panic!("invalid sexp"),
    }
}

pub fn into_ast(env: &mut InterpreterEnv, sexp: &Sexp) -> Result<Expr> {
    match sexp {
        Sexp::List(list) => {
            let (op, args) = (&list[0], list[1..].to_vec());
            let op = op.string()?;
            match op.as_str() {
                // (lam ((: x int)) x)
                "lam" => {
                    let params = args[0]
                        .list()?
                        .iter()
                        .map(|s| into_variable(env, s))
                        .collect::<Result<Vec<_>>>()?;
                    let body = args[1].clone();
                    Ok(Expr::FnDef(crate::ast::FnDef::new(
                        params,
                        Box::new(into_ast(env, &body)?),
                    )))
                }
                // (let a int 1)
                "let" => {
                    let (name, typ, val) = (args[0].string()?, args[1].string()?, args[2].clone());
                    let typ = if args.len() == 3 {
                        Some(Type::from(&env.alloc, typ)?)
                    } else {
                        None
                    };
                    let val = into_ast(env, &val)?;
                    Ok(Expr::Let(crate::ast::Let::new(
                        name.clone(),
                        typ,
                        Box::new(val),
                    )))
                }
                // reserved op redirects to function application
                "app" | "+" => {
                    let fun = into_ast(env, &args[0])?;
                    let args = args[1..]
                        .to_vec()
                        .iter()
                        .map(|s| into_ast(env, s))
                        .collect::<Result<Vec<_>>>()?;
                    Ok(Expr::FnApp(crate::ast::FnApp::new(Box::new(fun), args)))
                }
                _ => Err(anyhow::anyhow!("unknown op")),
            }
        }
        Sexp::String(lit) => {
            if is_number(&lit) {
                Ok(Expr::Literal(lit.parse::<i64>().unwrap()))
            } else {
                Ok(Expr::Variable(lit.to_string()))
            }
        }
        _ => panic!("invalid sexp"),
    }
}
