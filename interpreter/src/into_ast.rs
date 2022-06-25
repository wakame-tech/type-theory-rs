use std::collections::HashSet;

use hindley_milner::infer::analyse;
use structural_typesystem::types::Type;
use symbolic_expressions::Sexp;

use anyhow::Result;

use crate::{
    ast::{Expr, Parameter, Value},
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
                Ok(Parameter::new(name.clone(), typ.id()))
            }
        }
        _ => panic!("invalid sexp"),
    }
}

pub fn into_ast(env: &mut InterpreterEnv, sexp: &Sexp) -> Result<Expr> {
    match sexp {
        Sexp::List(list) => {
            let (op, args) = (&list[0], list[1..].to_vec());
            println!("[into_ast] op={} args={:?}", op, &args);
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
                    let (name, typ_id, val) = if args.len() == 3 {
                        let (name, typ, val) =
                            (args[0].string()?, args[1].string()?, args[2].clone());
                        let typ = Type::from(&env.alloc, typ)?;
                        println!("let {}: {} = {}", &name, &typ, &val);
                        (name, typ.id(), val)
                    } else {
                        let (name, val) = (args[0].string()?, args[1].clone());
                        println!("let {} = {}", &name, &val);
                        let typ_id = analyse(&mut env.alloc, &val, &env.type_env, &HashSet::new())?;
                        println!("{} inferred type #{}", name, typ_id);
                        (name, typ_id, val)
                    };
                    let val = into_ast(env, &val)?;
                    env.new_var(name.clone(), typ_id, Value::Nil);
                    Ok(Expr::Let(crate::ast::Let::new(
                        name.clone(),
                        typ_id,
                        Box::new(val),
                    )))
                }
                "app" => {
                    let name = args[0].string()?;
                    let args = args[1..]
                        .to_vec()
                        .iter()
                        .map(|s| into_ast(env, s))
                        .collect::<Result<Vec<_>>>()?;
                    Ok(Expr::FnApp(crate::ast::FnApp::new(
                        Box::new(Expr::Variable(name.clone())),
                        args,
                    )))
                }
                // reserved op redirects to function application
                "+" | "=" => {
                    let args = args
                        .to_vec()
                        .iter()
                        .map(|s| into_ast(env, s))
                        .collect::<Result<Vec<_>>>()?;
                    Ok(Expr::FnApp(crate::ast::FnApp::new(
                        Box::new(Expr::Variable(op.clone())),
                        args,
                    )))
                }
                _ => Err(anyhow::anyhow!("unknown op: \"{}\"", op)),
            }
        }
        Sexp::String(lit) => {
            if is_number(&lit) {
                Ok(Expr::Literal(Value::Int(lit.parse::<i64>().unwrap())))
            } else if lit == "true" || lit == "false" {
                Ok(Expr::Literal(Value::Bool(lit == "true")))
            } else {
                Ok(Expr::Variable(lit.to_string()))
            }
        }
        _ => panic!("invalid sexp"),
    }
}

#[cfg(test)]
mod tests {
    use super::into_ast;
    use crate::{
        ast::{Expr, Parameter, Value},
        interpreter_env::InterpreterEnv,
    };
    use anyhow::Result;
    use symbolic_expressions::parser::parse_str;

    fn should_be_ast(sexp: &str, expected: &Expr) -> Result<()> {
        let mut env = InterpreterEnv::new();
        let sexp = parse_str(sexp)?;
        let ast = into_ast(&mut env, &sexp).unwrap();
        assert_eq!(&ast, expected);
        Ok(())
    }

    #[test]
    fn int_literal() -> Result<()> {
        should_be_ast("1", &Expr::Literal(Value::Int(1)))
    }

    #[test]
    fn bool_literal() -> Result<()> {
        should_be_ast("true", &Expr::Literal(Value::Bool(true)))
    }

    #[test]
    fn var_literal() -> Result<()> {
        should_be_ast("x", &Expr::Variable("x".to_string()))
    }

    #[test]
    fn let_expr() -> Result<()> {
        should_be_ast(
            "(let x int 1)",
            &Expr::Let(crate::ast::Let::new(
                "x".to_string(),
                0,
                Box::new(Expr::Literal(Value::Int(1))),
            )),
        )
    }

    #[test]
    fn let_wo_anno() -> Result<()> {
        should_be_ast(
            "(let x 1)",
            &Expr::Let(crate::ast::Let::new(
                "x".to_string(),
                0,
                Box::new(Expr::Literal(Value::Int(1))),
            )),
        )
    }

    #[test]
    fn lam() -> Result<()> {
        should_be_ast(
            "(lam ((: x int)) x)",
            &Expr::FnDef(crate::ast::FnDef::new(
                vec![Parameter::new("x".to_string(), 0)],
                Box::new(Expr::Variable("x".to_string())),
            )),
        )
    }

    #[test]
    fn app() -> Result<()> {
        should_be_ast(
            "(app succ 1)",
            &Expr::FnApp(crate::ast::FnApp::new(
                Box::new(Expr::Variable("succ".to_string())),
                vec![Expr::Literal(Value::Int(1))],
            )),
        )
    }

    #[test]
    fn op_redirects_app() -> Result<()> {
        should_be_ast(
            "(+ 1 2)",
            &Expr::FnApp(crate::ast::FnApp::new(
                Box::new(Expr::Variable("+".to_string())),
                vec![Expr::Literal(Value::Int(1)), Expr::Literal(Value::Int(2))],
            )),
        )
    }
}
