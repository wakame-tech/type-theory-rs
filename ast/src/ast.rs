use anyhow::Result;
use std::fmt::{Debug, Display};
use symbolic_expressions::Sexp;

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub typ: Sexp,
}

impl Parameter {
    pub fn new(name: String, typ: Sexp) -> Self {
        Self { name, typ }
    }
}

impl Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.typ)
    }
}

/// (f a)
#[derive(Debug, Clone, PartialEq)]
pub struct FnApp(pub Box<Expr>, pub Box<Expr>);

impl FnApp {
    pub fn new(f: Expr, value: Expr) -> Self {
        Self(Box::new(f), Box::new(value))
    }
}

impl Display for FnApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {})", self.0, self.1)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnDef {
    pub param: Parameter,
    pub body: Box<Expr>,
}

impl FnDef {
    pub fn new(param: Parameter, body: Box<Expr>) -> Self {
        Self { param, body }
    }
}

impl Display for FnDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}) -> {}", self.param.to_string(), self.body)
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
            "let {}: {} = {};",
            self.name,
            self.typ.as_ref().unwrap_or(&Sexp::Empty),
            self.value
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    pub raw: Sexp,
}

impl Value {
    pub fn new(raw: Sexp) -> Self {
        Value { raw }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}

pub fn from_expr(expr: &Expr) -> Result<Value> {
    match expr {
        Expr::Literal(v) => Ok(v.clone()),
        _ => Err(anyhow::anyhow!("{} is not value", expr)),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MacroApp(pub Sexp);

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Value),
    Variable(String),
    Let(Let),
    FnApp(FnApp),
    FnDef(FnDef),
    MacroApp(MacroApp),
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
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Literal(literal) => write!(f, "{}", literal),
            Expr::Variable(name) => write!(f, "{}", name),
            Expr::Let(let_) => write!(f, "{}", let_),
            Expr::FnApp(fn_app) => write!(f, "{}", fn_app),
            Expr::FnDef(fn_def) => write!(f, "{}", fn_def),
            Expr::MacroApp(macro_app) => write!(f, "{}", macro_app.0),
        }
    }
}

#[derive(Debug)]
pub struct Program(pub Vec<Expr>);
