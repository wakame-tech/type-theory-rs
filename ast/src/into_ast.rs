use crate::ast::{Expr, FnDef, Let, Parameter, Value};
use anyhow::Result;
use structural_typesystem::{type_alloc::TypeAlloc, types::Id};
use symbolic_expressions::Sexp;

fn is_number(s: &str) -> bool {
    s.chars().all(|c| c.is_numeric())
}

pub fn parse_type(alloc: &mut TypeAlloc, type_sexp: &Sexp) -> Result<Id> {
    let Ok(list) = type_sexp.list() else {
        return Err(anyhow::anyhow!("type annotation expected"));
    };
    if list[0].string()? != ":" {
        return Err(anyhow::anyhow!("type annotation expected"));
    }
    alloc.from_sexp(&list[1])
}

/// parse (x (: int))
pub fn parse_parameter(alloc: &mut TypeAlloc, sexp: &Sexp) -> Result<Parameter> {
    let Ok(list) = sexp.list() else {
        return Err(anyhow::anyhow!("parameter must be list"));
    };
    Ok(Parameter::new(
        list[0].string()?.to_string(),
        parse_type(alloc, &list[1])?,
    ))
}

pub fn into_ast(alloc: &mut TypeAlloc, sexp: &Sexp) -> Result<Expr> {
    match sexp {
        Sexp::List(list) => {
            let (op, args) = (&list[0], list[1..].to_vec());
            println!("[into_ast] op={} args={:?}", op, &args);
            let op = op.string()?;
            match op.as_str() {
                // (lam (x (: int)) x)
                "lam" => {
                    let param = parse_parameter(alloc, &args[0])?;
                    let body = args[1].clone();
                    let body_ast = Box::new(into_ast(alloc, &body)?);
                    Ok(Expr::FnDef(FnDef::new(alloc, vec![param], body_ast)))
                }
                // (let a (: int) 1) or (let a 1)
                "let" => {
                    let let_node = match args.len() {
                        2 => {
                            let (name, val) = (args[0].string()?, args[1].clone());
                            println!("let {} = {}", &name, &val);
                            let val = into_ast(alloc, &val)?;
                            Let::new(name.to_string(), val.type_id(), Box::new(val))
                        }
                        3 => {
                            let (name, type_sexp, val) =
                                (args[0].string()?, &args[1], args[2].clone());
                            let typ = parse_type(alloc, type_sexp)?;
                            let val = into_ast(alloc, &val)?;
                            Let::new(name.to_string(), typ, Box::new(val))
                        }
                        _ => panic!("invalid let"),
                    };
                    Ok(Expr::Let(let_node))
                }
                // (f 1)
                _ => {
                    let args = args
                        .iter()
                        .map(|s| into_ast(alloc, s))
                        .collect::<Result<Vec<_>>>()?;
                    Ok(Expr::FnApp(crate::ast::FnApp::new(
                        alloc,
                        Box::new(Expr::Variable(op.clone())),
                        args,
                    )))
                }
            }
        }
        Sexp::String(lit) => {
            if is_number(&lit) {
                let raw = lit.parse::<i64>()?;
                let type_id = alloc.from("int")?;
                Ok(Expr::Literal(Value {
                    raw: raw.to_string(),
                    type_id,
                }))
            } else if lit == "true" || lit == "false" {
                let type_id = alloc.from("bool")?;
                Ok(Expr::Literal(Value {
                    raw: lit.to_string(),
                    type_id,
                }))
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
    use crate::ast::{Expr, FnApp, FnDef, Let, Parameter, Value};
    use anyhow::Result;
    use structural_typesystem::{type_alloc::TypeAlloc, type_env::setup_type_env};
    use symbolic_expressions::parser::parse_str;

    fn should_be_ast(alloc: &mut TypeAlloc, sexp: &str, expected: &Expr) -> Result<()> {
        let sexp = parse_str(sexp)?;
        let ast = into_ast(alloc, &sexp).unwrap();
        assert_eq!(&ast, expected);
        Ok(())
    }

    #[test]
    fn int_literal() -> Result<()> {
        let (mut env, mut alloc) = setup_type_env()?;
        let type_id = alloc.from("int")?;
        should_be_ast(
            &mut alloc,
            "1",
            &Expr::Literal(Value {
                raw: "1".to_string(),
                type_id,
            }),
        )
    }

    #[test]
    fn bool_literal() -> Result<()> {
        let (mut env, mut alloc) = setup_type_env()?;
        let type_id = alloc.from("bool")?;
        should_be_ast(
            &mut alloc,
            "true",
            &Expr::Literal(Value {
                raw: "true".to_string(),
                type_id,
            }),
        )
    }

    #[test]
    fn var_literal() -> Result<()> {
        let (mut env, mut alloc) = setup_type_env()?;
        should_be_ast(&mut alloc, "x", &Expr::Variable("x".to_string()))
    }

    #[test]
    fn let_expr() -> Result<()> {
        let (mut env, mut alloc) = setup_type_env()?;
        let type_id = alloc.from("int")?;
        should_be_ast(
            &mut alloc,
            "(let x (: int) 1)",
            &Expr::Let(Let::new(
                "x".to_string(),
                type_id,
                Box::new(Expr::Literal(Value {
                    raw: "1".to_string(),
                    type_id,
                })),
            )),
        )
    }

    #[test]
    fn let_wo_anno() -> Result<()> {
        let (mut env, mut alloc) = setup_type_env()?;
        let type_id = alloc.from("int")?;

        should_be_ast(
            &mut alloc,
            "(let x 1)",
            &Expr::Let(Let::new(
                "x".to_string(),
                type_id,
                Box::new(Expr::Literal(Value {
                    raw: "1".to_string(),
                    type_id,
                })),
            )),
        )
    }

    #[test]
    fn lam() -> Result<()> {
        let (mut env, mut alloc) = setup_type_env()?;
        let type_id = alloc.from("int")?;

        let fn_def = Expr::FnDef(FnDef::new(
            &mut alloc,
            vec![Parameter::new("x".to_string(), type_id)],
            Box::new(Expr::Variable("x".to_string())),
        ));

        should_be_ast(&mut alloc, "(lam (x (: int)) x)", &fn_def)
    }

    #[test]
    fn lam_wo_anno() -> Result<()> {
        let (mut env, mut alloc) = setup_type_env()?;
        let type_id = alloc.from("int")?;

        let fn_def = Expr::FnDef(FnDef::new(
            &mut alloc,
            vec![Parameter::new("x".to_string(), type_id)],
            Box::new(Expr::Variable("x".to_string())),
        ));

        should_be_ast(&mut alloc, "(lam (x (: int)) x)", &fn_def)
    }

    #[test]
    fn app() -> Result<()> {
        let (mut env, mut alloc) = setup_type_env()?;
        let type_id = alloc.from("int")?;
        let fn_app = Expr::FnApp(FnApp::new(
            &mut alloc,
            Box::new(Expr::Variable("succ".to_string())),
            vec![Expr::Literal(Value {
                raw: "1".to_string(),
                type_id,
            })],
        ));

        should_be_ast(&mut alloc, "(succ 1)", &fn_app)
    }

    #[test]
    fn op_redirects_app() -> Result<()> {
        let (mut env, mut alloc) = setup_type_env()?;
        let type_id = alloc.from("int")?;

        let fn_app = Expr::FnApp(crate::ast::FnApp::new(
            &mut alloc,
            Box::new(Expr::Variable("+".to_string())),
            vec![
                Expr::Literal(Value {
                    raw: "1".to_string(),
                    type_id,
                }),
                Expr::Literal(Value {
                    raw: "2".to_string(),
                    type_id,
                }),
            ],
        ));

        should_be_ast(&mut alloc, "(+ 1 2)", &fn_app)
    }
}
