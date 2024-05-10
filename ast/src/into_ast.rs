use crate::ast::{Expr, FnApp, FnDef, Let, MacroApp, Parameter, Value};
use anyhow::Result;
use std::collections::HashMap;
use symbolic_expressions::Sexp;

fn parse_parameter(sexp: &Sexp) -> Result<Parameter> {
    match sexp {
        // without type annotation: `a`
        Sexp::String(arg) => Ok(Parameter::new(arg.to_string(), None)),
        // with type annotation: `(a : int)`
        Sexp::List(list) if list.len() == 3 && list[1].string().ok() == Some(&":".to_string()) => {
            let name = list[0].string()?;
            let typ = list[2].clone();
            Ok(Parameter::new(name.to_string(), Some(typ)))
        }
        _ => Err(anyhow::anyhow!(
            "parameter must be a string or a list. but {}",
            sexp
        )),
    }
}

/// (lam (x : int) x)
fn parse_lambda(list: &[Sexp]) -> Result<Expr> {
    let param = parse_parameter(&list[1])?;
    let body = Box::new(into_ast(&list[2])?);
    Ok(Expr::FnDef(FnDef::new(param, body)))
}

fn parse_let(sexp: &Sexp) -> Result<Expr> {
    let list = sexp.list()?;
    match list.len() {
        // without type annotation: `(let a 1)`
        3 => Ok(Expr::Let(Let::new(
            list[1].string()?.to_string(),
            None,
            Box::new(into_ast(&list[2])?),
        ))),
        // with type annotation: `(let a : int 1)`
        5 if list[2].string().ok() == Some(&":".to_string()) => Ok(Expr::Let(Let::new(
            list[1].string()?.to_string(),
            Some(list[3].clone()),
            Box::new(into_ast(&list[4])?),
        ))),
        _ => Err(anyhow::anyhow!(
            "let must have 2 or 3 operands. but {}",
            sexp
        )),
    }
}

/// (f g h) -> ((f g) h)
fn parse_apply(f: &Sexp, v: &Sexp) -> Result<Expr> {
    let (f, v) = (into_ast(f)?, into_ast(v)?);
    Ok(Expr::FnApp(FnApp::new(f, v)))
}

fn parse_record(entries: &[Sexp]) -> Result<Value> {
    let mut res = HashMap::new();
    for entry in entries {
        let entry = entry.list()?;
        let key = entry[0].string()?;
        let value = into_ast(&entry[1])?;
        res.insert(key.to_string(), value);
    }
    Ok(Value::Record(res))
}

fn is_number(s: &str) -> bool {
    s.chars().all(|c| c.is_numeric())
}

pub fn into_ast(sexp: &Sexp) -> Result<Expr> {
    match sexp {
        Sexp::List(list) => match list[0] {
            Sexp::String(ref lam) if lam == "lam" => parse_lambda(list),
            Sexp::String(ref lt) if lt == "let" => parse_let(sexp),
            _ if list[0].is_string() && list[0].string()? == &"record".to_string() => {
                Ok(Expr::Literal(parse_record(&list[1..])?))
            }
            _ if list[0].is_string() && list[0].string()?.ends_with('!') => {
                Ok(Expr::MacroApp(MacroApp(Sexp::List(
                    [vec![Sexp::String(list[0].string()?.to_string())],
                        list[1..].to_vec()]
                    .concat(),
                ))))
            }
            _ if list.len() == 2 => parse_apply(&list[0], &list[1]),
            _ => Err(anyhow::anyhow!("illegal operands")),
        },
        Sexp::String(lit) => match lit.as_str() {
            "nil" => Ok(Expr::Literal(Value::Nil)),
            _ if is_number(lit) => Ok(Expr::Literal(Value::Number(lit.parse()?))),
            "true" | "false" => Ok(Expr::Literal(Value::Bool(lit.parse()?))),
            _ => Ok(Expr::Variable(lit.to_string())),
        },
        _ => Err(anyhow::anyhow!("invalid sexp: {}", sexp)),
    }
}

#[cfg(test)]
mod tests {
    use super::{into_ast, parse_parameter};
    use crate::ast::{Expr, FnApp, FnDef, Let, Parameter, Value};
    use anyhow::Result;
    use std::collections::HashMap;
    use symbolic_expressions::{parser::parse_str, Sexp};

    fn should_be_ast(sexp: &str, expected: &Expr) -> Result<()> {
        let sexp = parse_str(sexp)?;
        let ast = into_ast(&sexp).unwrap();
        assert_eq!(&ast, expected);
        Ok(())
    }

    #[test]
    fn nil_literal() -> Result<()> {
        should_be_ast("nil", &Expr::Literal(Value::Nil))
    }

    #[test]
    fn int_literal() -> Result<()> {
        should_be_ast("1", &Expr::Literal(Value::Number(1)))
    }

    #[test]
    fn bool_literal() -> Result<()> {
        should_be_ast("true", &Expr::Literal(Value::Bool(true)))
    }

    #[test]
    fn record_literal() -> Result<()> {
        should_be_ast(
            "(record (a 1) (b 2))",
            &Expr::Literal(Value::Record(HashMap::from_iter(vec![
                ("a".to_string(), Expr::Literal(Value::Number(1))),
                ("b".to_string(), Expr::Literal(Value::Number(2))),
            ]))),
        )
    }

    #[test]
    fn var_literal() -> Result<()> {
        should_be_ast("x", &Expr::Variable("x".to_string()))
    }

    #[test]
    fn parameter() -> Result<()> {
        let param = parse_parameter(&parse_str("(a : int)")?)?;
        assert_eq!(
            param,
            Parameter::new("a".to_string(), Some(Sexp::String("int".to_string())))
        );
        let param = parse_parameter(&parse_str("a")?)?;
        assert_eq!(param, Parameter::new("a".to_string(), None));
        Ok(())
    }

    #[test]
    fn let_expr() -> Result<()> {
        should_be_ast(
            "(let x : int 1)",
            &Expr::Let(Let::new(
                "x".to_string(),
                Some(Sexp::String("int".to_string())),
                Box::new(Expr::Literal(Value::Number(1))),
            )),
        )
    }

    #[test]
    fn let_wo_anno() -> Result<()> {
        should_be_ast(
            "(let x 1)",
            &Expr::Let(Let::new(
                "x".to_string(),
                None,
                Box::new(Expr::Literal(Value::Number(1))),
            )),
        )
    }

    #[test]
    fn lam() -> Result<()> {
        let fn_def = Expr::FnDef(FnDef::new(
            Parameter::new("x".to_string(), Some(Sexp::String("int".to_string()))),
            Box::new(Expr::Variable("x".to_string())),
        ));
        should_be_ast("(lam (x : int) x)", &fn_def)
    }

    #[test]
    fn lam_wo_anno() -> Result<()> {
        let fn_def = Expr::FnDef(FnDef::new(
            Parameter::new("x".to_string(), None),
            Box::new(Expr::Variable("x".to_string())),
        ));
        should_be_ast("(lam x x)", &fn_def)
    }

    #[test]
    fn app() -> Result<()> {
        let fn_app = Expr::FnApp(FnApp::new(
            Expr::Variable("succ".to_string()),
            Expr::Literal(Value::Number(1)),
        ));
        should_be_ast("(succ 1)", &fn_app)
    }

    #[test]
    fn op_redirects_app() -> Result<()> {
        // (+ 1 2) -> ((+ 1) 2)
        let plus1 = Expr::FnApp(FnApp::new(
            Expr::Variable("+".to_string()),
            Expr::Literal(Value::Number(1)),
        ));
        let plus1_2 = Expr::FnApp(FnApp::new(plus1, Expr::Literal(Value::Number(2))));
        should_be_ast("((+ 1) 2)", &plus1_2)
    }
}
