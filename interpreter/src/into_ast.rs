use structural_typesystem::types::Type;
use symbolic_expressions::Sexp;

use anyhow::Result;

use crate::{
    ast::{Expr, Variable},
    env::Env,
};

fn is_number(s: &str) -> bool {
    s.chars().all(|c| c.is_numeric())
}

pub fn into_variable(env: &Env, sexp: &Sexp) -> Result<Variable> {
    match sexp {
        // with type annotation
        Sexp::List(list) => {
            if list[0].string()? != ":" {
                Err(anyhow::anyhow!("type annotation expected"))
            } else {
                let (name, annotation) = (list[1].string()?, list[2].string()?);
                let typ = Type::from(&env.alloc, &annotation)?;
                Ok(Variable::new(name.clone(), Some(typ)))
            }
        }
        // without type annotation
        Sexp::String(name) => {
            if is_number(name) {
                return Err(anyhow::anyhow!("variable expected"));
            } else {
                Ok(Variable::new(name.to_string(), None))
            }
        }
        _ => panic!("invalid sexp"),
    }
}

pub fn into_ast(env: &Env, sexp: &Sexp) -> Result<Expr> {
    match sexp {
        Sexp::List(list) => {
            let (op, args) = (&list[0], list[1..].to_vec());
            let op = op.string()?;
            match op.as_str() {
                "lam" => {
                    let name = args[0].string()?;
                    let params = args[1..]
                        .to_vec()
                        .iter()
                        .map(|s| into_variable(env, s))
                        .collect::<Result<Vec<_>>>()?;
                    let body = args[2].clone();
                    Ok(Expr::FnDef(crate::ast::FnDef::new(
                        name.clone(),
                        params,
                        Box::new(into_ast(env, &body)?),
                    )))
                }
                "app" => {
                    let name = args[0].string()?;
                    let args = args[1..]
                        .to_vec()
                        .iter()
                        .map(|s| into_ast(env, s))
                        .collect::<Result<Vec<_>>>()?;
                    Ok(Expr::FnApp(crate::ast::FnApp::new(name.clone(), args)))
                }
                "+" => {
                    let (lhs, rhs) = (into_ast(env, &args[0])?, into_ast(env, &args[1])?);
                    Ok(Expr::FnApp(crate::ast::FnApp::new(
                        op.clone(),
                        vec![lhs, rhs],
                    )))
                }
                _ => Err(anyhow::anyhow!("Unknown op")),
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
