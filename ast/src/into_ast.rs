use crate::ast::{Expr, FnApp, FnDef, Let, MacroApp, Parameter, Value};
use anyhow::Result;
use structural_typesystem::{type_alloc::TypeAlloc, types::Id};
use symbolic_expressions::{parser::parse_str, Sexp};

fn is_number(s: &str) -> bool {
    s.chars().all(|c| c.is_numeric())
}

fn is_bool(s: &str) -> bool {
    s == "true" || s == "false"
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

/// parse (x : int)
pub fn parse_parameter(sexp: &Sexp) -> Result<Parameter> {
    let Ok(list) = sexp.list() else {
        return Err(anyhow::anyhow!("parameter must be list"));
    };
    anyhow::ensure!(list[1].string()? == ":");
    Ok(Parameter::new(
        list[0].string()?.to_string(),
        list[2].clone(),
    ))
}

/// (lam (x : int) x)
pub fn parse_lambda(list: &[Sexp]) -> Result<Expr> {
    let param = parse_parameter(&list[1])?;
    let body = Box::new(into_ast(&list[2])?);
    log::debug!("parse_lambda {} {}", param, body);
    Ok(Expr::FnDef(FnDef::new(param, body)))
}

///
/// - with type annotation: `(let a int 1)`
/// - without type annotation: `(let a 1)`
pub fn parse_let(list: &[Sexp]) -> Result<Expr> {
    let let_node = match list.len() {
        3 => {
            let (name, val) = (list[1].string()?, &list[2]);
            let val = into_ast(val)?;
            Let::new(name.to_string(), None, Box::new(val))
        }
        4 => {
            let (name, typ, val) = (list[1].string()?, &list[2], &list[3]);
            log::debug!("{} {} {}", name, typ, val);
            let val = into_ast(val)?;
            Let::new(name.to_string(), Some(typ.clone()), Box::new(val))
        }
        _ => panic!("let/3 nor let/4"),
    };
    Ok(Expr::Let(let_node))
}

pub fn reduce<T, F>(a: Result<T>, b: Result<T>, f: F) -> Result<T>
where
    F: FnOnce(T, T) -> T,
{
    match (a, b) {
        (Ok(l), Ok(r)) => Ok(f(l, r)),
        (Ok(_), Err(e)) | (Err(e), Ok(_)) => Err(e),
        (Err(e1), Err(e2)) => Err(anyhow::anyhow!("{}, {}", e1, e2)),
    }
}

/// (f g h) -> ((f g) h)
pub fn parse_apply(f: &Sexp, v: &Sexp) -> Result<Expr> {
    let (f, v) = (into_ast(f)?, into_ast(v)?);
    Ok(Expr::FnApp(FnApp::new(f, v)))
}

pub fn into_ast(sexp: &Sexp) -> Result<Expr> {
    match sexp {
        Sexp::List(list) => match list[0] {
            Sexp::String(ref lam) if lam == "lam" => parse_lambda(list),
            Sexp::String(ref lt) if lt == "let" => parse_let(list),
            _ if list[0].string()?.ends_with('!') => Ok(Expr::MacroApp(MacroApp(Sexp::List(
                vec![
                    vec![Sexp::String(list[0].string()?.to_string())],
                    list[1..].to_vec(),
                ]
                .concat(),
            )))),
            _ if list.len() == 2 => parse_apply(&list[0], &list[1]),
            _ => Err(anyhow::anyhow!("illegal operands")),
        },
        Sexp::String(lit) => match lit.as_str() {
            _ if is_number(lit) => Ok(Expr::Literal(Value {
                raw: parse_str(lit)?,
            })),
            _ if is_bool(lit) => Ok(Expr::Literal(Value {
                raw: parse_str(lit)?,
            })),
            _ => Ok(Expr::Variable(lit.to_string())),
        },
        _ => panic!("invalid sexp"),
    }
}

#[cfg(test)]
mod tests {
    use super::{into_ast, parse_parameter};
    use crate::ast::{Expr, FnApp, FnDef, Let, Parameter, Value};
    use anyhow::Result;
    use symbolic_expressions::{parser::parse_str, Sexp};

    fn make_value(value: &str) -> Result<Value> {
        Ok(Value {
            raw: parse_str(value)?,
        })
    }

    fn should_be_ast(sexp: &str, expected: &Expr) -> Result<()> {
        let sexp = parse_str(sexp)?;
        let ast = into_ast(&sexp).unwrap();
        assert_eq!(&ast, expected);
        Ok(())
    }

    #[test]
    fn int_literal() -> Result<()> {
        let v = make_value("1")?;
        should_be_ast("1", &Expr::Literal(v))
    }

    #[test]
    fn bool_literal() -> Result<()> {
        let value = make_value("true")?;
        should_be_ast("true", &Expr::Literal(value))
    }

    #[test]
    fn var_literal() -> Result<()> {
        should_be_ast("x", &Expr::Variable("x".to_string()))
    }

    #[test]
    fn parameter() -> Result<()> {
        let param = parse_parameter(&parse_str("(: a int)")?)?;
        assert_eq!(
            param,
            Parameter::new("a".to_string(), Sexp::String("int".to_string()))
        );
        Ok(())
    }

    #[test]
    fn let_expr() -> Result<()> {
        let value = make_value("1")?;
        should_be_ast(
            "(let x (: int) 1)",
            &Expr::Let(Let::new(
                "x".to_string(),
                Some(Sexp::String("int".to_string())),
                Box::new(Expr::Literal(value)),
            )),
        )
    }

    #[test]
    fn let_wo_anno() -> Result<()> {
        let value = make_value("1")?;
        should_be_ast(
            "(let x 1)",
            &Expr::Let(Let::new(
                "x".to_string(),
                Some(Sexp::String("int".to_string())),
                Box::new(Expr::Literal(value)),
            )),
        )
    }

    #[test]
    fn lam() -> Result<()> {
        let fn_def = Expr::FnDef(FnDef::new(
            Parameter::new("x".to_string(), Sexp::String("int".to_string())),
            Box::new(Expr::Variable("x".to_string())),
        ));
        should_be_ast("(lam (x (: int)) x)", &fn_def)
    }

    #[test]
    fn lam_wo_anno() -> Result<()> {
        let fn_def = Expr::FnDef(FnDef::new(
            Parameter::new("x".to_string(), Sexp::String("int".to_string())),
            Box::new(Expr::Variable("x".to_string())),
        ));
        should_be_ast("(lam (x (: int)) x)", &fn_def)
    }

    #[test]
    fn app() -> Result<()> {
        let value = make_value("1")?;
        let fn_app = Expr::FnApp(FnApp::new(
            Expr::Variable("succ".to_string()),
            Expr::Literal(value),
        ));
        should_be_ast("(succ 1)", &fn_app)
    }

    #[test]
    fn op_redirects_app() -> Result<()> {
        let value1 = make_value("1")?;
        let value2 = make_value("2")?;

        // (+ 1 2) -> ((+ 1) 2)
        let plus1 = Expr::FnApp(FnApp::new(
            Expr::Variable("+".to_string()),
            Expr::Literal(value1),
        ));
        let plus1_2 = Expr::FnApp(FnApp::new(plus1, Expr::Literal(value2)));
        should_be_ast("((+ 1) 2)", &plus1_2)
    }
}
