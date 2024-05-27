use crate::ast::{Case, Expr, FnApp, FnDef, Let, Parameter, TypeDef, Value};
use anyhow::Result;
use std::collections::HashMap;
use symbolic_expressions::Sexp;

pub const LET_KEYWORD: &str = "let";
pub const FN_KEYWORD: &str = "fn";
pub const RECORD_KEYWORD: &str = "record";
pub const LIST_KEYWORD: &str = "vec";
pub const TYPE_KEYWORD: &str = "type";
pub const CASE_KEYWORD: &str = "case";
pub const EXTERNAL_KEYWORD: &str = "external";
pub const INCLUDE_KEYWORD: &str = "include";

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

/// (fn (x : int) x)
fn parse_fn(list: &[Sexp]) -> Result<Expr> {
    let params = list[1..list.len() - 1]
        .iter()
        .map(parse_parameter)
        .collect::<Result<Vec<_>>>()?;
    let body = Box::new(into_ast(&list[list.len() - 1])?);
    Ok(Expr::FnDef(FnDef::new(params, body)))
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

fn parse_type(sexp: &Sexp) -> Result<Expr> {
    let list = sexp.list()?;
    anyhow::ensure!(
        list[2].is_string() && list[2].string()? == ":",
        "missing colon in {}",
        sexp
    );
    let (name, typ) = (list[1].string()?, list[3].clone());
    Ok(Expr::TypeDef(TypeDef::new(name.to_string(), typ)))
}

/// (f g h) -> ((f g) h)
fn parse_apply(f: &Sexp, values: &[Sexp]) -> Result<Expr> {
    let f = into_ast(f)?;
    let v = values.iter().map(into_ast).collect::<Result<Vec<_>>>()?;
    Ok(Expr::FnApp(FnApp::new(f, v)))
}

fn parse_record(entries: &[Sexp]) -> Result<Value> {
    let mut res = HashMap::new();
    for entry in entries {
        let entry = entry.list()?;
        let key = entry[0].string()?;
        anyhow::ensure!(entry[1].string()? == ":", "missing colon {:?}", entry);
        let value = into_ast(&entry[2])?;
        res.insert(key.to_string(), value);
    }
    Ok(Value::Record(res))
}

fn parse_list(elements: &[Sexp]) -> Result<Value> {
    let elements = elements.iter().map(into_ast).collect::<Result<Vec<_>>>()?;
    Ok(Value::List(elements))
}

fn is_number(s: &str) -> bool {
    s.chars().all(|c| c.is_numeric())
}

pub fn parse_case(branches: &[Sexp]) -> Result<Expr> {
    let branches = branches
        .iter()
        .map(|branch| {
            let branch = branch.list()?;
            let pattern = into_ast(&branch[0])?;
            anyhow::ensure!(
                branch[1].is_string() && branch[1].string()? == "=>",
                "missing =>"
            );
            let body = into_ast(&branch[2])?;
            Ok((pattern, body))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Expr::Case(Case::new(branches)))
}

pub fn into_ast(sexp: &Sexp) -> Result<Expr> {
    let _span = tracing::debug_span!("", "{}", sexp).entered();
    let expr = match sexp {
        Sexp::List(list) => match list[0] {
            Sexp::String(ref head) if head == FN_KEYWORD => parse_fn(list),
            Sexp::String(ref head) if head == LET_KEYWORD => parse_let(sexp),
            Sexp::String(ref head) if head == TYPE_KEYWORD => parse_type(sexp),
            Sexp::String(ref head) if head == CASE_KEYWORD => parse_case(&list[1..]),
            Sexp::String(ref head) if head == INCLUDE_KEYWORD => {
                Ok(Expr::Include(list[1].string()?.to_string()))
            }
            _ if list[0].is_string() && list[0].string()?.as_str() == EXTERNAL_KEYWORD => Ok(
                Expr::Literal(Value::External(list[1].string()?.to_string())),
            ),
            _ if list[0].is_string() && list[0].string()?.as_str() == RECORD_KEYWORD => {
                Ok(Expr::Literal(parse_record(&list[1..])?))
            }
            _ if list[0].is_string() && list[0].string()?.as_str() == LIST_KEYWORD => {
                Ok(Expr::Literal(parse_list(&list[1..])?))
            }
            _ => parse_apply(&list[0], &list[1..]),
        },
        Sexp::String(lit) => match lit.as_str() {
            _ if is_number(lit) => Ok(Expr::Literal(Value::Number(lit.parse()?))),
            _ if lit.starts_with('\'') && lit.ends_with('\'') => Ok(Expr::Literal(Value::String(
                lit[1..lit.len() - 1].to_string(),
            ))),
            "true" | "false" => Ok(Expr::Literal(Value::Bool(lit.parse()?))),
            _ if lit.starts_with(':') => Ok(Expr::Literal(Value::Atom(
                lit.trim_start_matches(':').to_string(),
            ))),
            _ => Ok(Expr::Variable(lit.to_string())),
        },
        _ => Err(anyhow::anyhow!("invalid sexp: {}", sexp)),
    }?;
    log::debug!("={}", expr);
    Ok(expr)
}

#[cfg(test)]
mod tests {
    use super::{into_ast, parse_parameter};
    use crate::ast::{Expr, FnApp, FnDef, Let, Parameter, TypeDef, Value};
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
    fn int_literal() -> Result<()> {
        should_be_ast("1", &Expr::Literal(Value::Number(1)))
    }

    #[test]
    fn bool_literal() -> Result<()> {
        should_be_ast("true", &Expr::Literal(Value::Bool(true)))
    }

    #[test]
    fn atom_literal() -> Result<()> {
        should_be_ast(":atom", &Expr::Literal(Value::Atom("atom".to_string())))
    }

    #[test]
    fn string_literal() -> Result<()> {
        should_be_ast("'str'", &Expr::Literal(Value::String("str".to_string())))
    }

    #[test]
    fn record_literal() -> Result<()> {
        should_be_ast(
            "(record (a : 1) (b : 2))",
            &Expr::Literal(Value::Record(HashMap::from_iter(vec![
                ("a".to_string(), Expr::Literal(Value::Number(1))),
                ("b".to_string(), Expr::Literal(Value::Number(2))),
            ]))),
        )
    }

    #[test]
    fn list_literal() -> Result<()> {
        should_be_ast(
            "(vec 1 2 3)",
            &Expr::Literal(Value::List(vec![
                Expr::Literal(Value::Number(1)),
                Expr::Literal(Value::Number(2)),
                Expr::Literal(Value::Number(3)),
            ])),
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
    fn fn_def() -> Result<()> {
        let fn_def = Expr::FnDef(FnDef::new(
            vec![Parameter::new(
                "x".to_string(),
                Some(Sexp::String("int".to_string())),
            )],
            Box::new(Expr::Variable("x".to_string())),
        ));
        should_be_ast("(fn (x : int) x)", &fn_def)
    }

    #[test]
    fn fn_wo_anno() -> Result<()> {
        let fn_def = Expr::FnDef(FnDef::new(
            vec![Parameter::new("x".to_string(), None)],
            Box::new(Expr::Variable("x".to_string())),
        ));
        should_be_ast("(fn x x)", &fn_def)
    }

    #[test]
    fn app() -> Result<()> {
        let fn_app = Expr::FnApp(FnApp::new(
            Expr::Variable("succ".to_string()),
            vec![Expr::Literal(Value::Number(1))],
        ));
        should_be_ast("(succ 1)", &fn_app)
    }

    #[test]
    fn type_def() -> Result<()> {
        let expr = Expr::TypeDef(TypeDef::new(
            "a".to_string(),
            Sexp::String("int".to_string()),
        ));
        should_be_ast("(type a : int)", &expr)
    }

    #[test]
    fn case() -> Result<()> {
        let expr = Expr::Case(crate::ast::Case::new(vec![
            (
                Expr::Literal(Value::Number(1)),
                Expr::Literal(Value::Number(2)),
            ),
            (
                Expr::Literal(Value::Number(3)),
                Expr::Literal(Value::Number(4)),
            ),
        ]));
        should_be_ast("(case (1 => 2) (3 => 4))", &expr)
    }
}
