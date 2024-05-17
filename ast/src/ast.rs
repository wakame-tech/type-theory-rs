use anyhow::Result;
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};
use symbolic_expressions::Sexp;

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub typ: Option<Sexp>,
}

impl Parameter {
    pub fn new(name: String, typ: Option<Sexp>) -> Self {
        Self { name, typ }
    }
}

impl Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(typ) = &self.typ {
            write!(f, "{}: {}", self.name, typ)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

/// (f a)
#[derive(Debug, Clone, PartialEq)]
pub struct FnApp(pub Box<Expr>, pub Vec<Box<Expr>>);

impl FnApp {
    pub fn new(f: Expr, values: Vec<Expr>) -> Self {
        Self(Box::new(f), values.into_iter().map(Box::new).collect())
    }
}

impl Display for FnApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({} {})",
            self.0,
            self.1
                .iter()
                .map(|v| format!("{}", v))
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnDef {
    pub args: Vec<Parameter>,
    pub body: Box<Expr>,
}

impl FnDef {
    pub fn new(args: Vec<Parameter>, body: Box<Expr>) -> Self {
        Self { args, body }
    }
}

impl Display for FnDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}) -> {}",
            self.args
                .iter()
                .map(|a| format!("{}", a))
                .collect::<Vec<String>>()
                .join(" "),
            self.body
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Let {
    pub name: String,
    pub typ: Option<Sexp>,
    pub value: Box<Expr>,
}

impl Let {
    pub fn new(name: String, typ: Option<Sexp>, value: Box<Expr>) -> Self {
        Self { name, typ, value }
    }
}

impl Display for Let {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "let {} : {} = {}",
            self.name,
            self.typ.as_ref().unwrap_or(&Sexp::Empty),
            self.value
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(i64),
    Record(HashMap<String, Expr>),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::Record(record) => write!(
                f,
                "(record {})",
                record
                    .iter()
                    .map(|(k, v)| format!("({} : {})", k, v))
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
        }
    }
}

pub fn from_expr(expr: &Expr) -> Result<Value> {
    match expr {
        Expr::Literal(v) => Ok(v.clone()),
        _ => Err(anyhow::anyhow!("{} is not value", expr)),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Value),
    Variable(String),
    Let(Let),
    FnApp(FnApp),
    FnDef(FnDef),
}

impl Expr {
    pub fn literal(self) -> Result<Value> {
        match self {
            Expr::Literal(literal) => Ok(literal),
            _ => Err(anyhow::anyhow!("literal expected")),
        }
    }

    pub fn name(self) -> Result<String> {
        match self {
            Expr::Variable(name) => Ok(name),
            _ => Err(anyhow::anyhow!("variable expected")),
        }
    }

    pub fn has_context(&self) -> bool {
        matches!(self, Expr::Let(_) | Expr::FnDef(_))
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Literal(literal) => write!(f, "{}", literal),
            Expr::Variable(name) => write!(f, "{}", name),
            Expr::Let(let_) => write!(f, "{}", let_),
            Expr::FnApp(fn_app) => write!(f, "{}", fn_app),
            Expr::FnDef(fn_def) => write!(f, "{}", fn_def),
        }
    }
}

#[derive(Debug)]
pub struct Program(pub Vec<Expr>);
