use std::fmt::Display;

use anyhow::Result;
use structural_typesystem::types::Type;

use crate::interpreter_env::InterpreterEnv;

pub trait Eval {
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr>;
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub typ: Type,
}

impl Parameter {
    pub fn new(name: String, typ: Type) -> Self {
        Self { name, typ }
    }
}

impl Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.typ)
    }
}

#[derive(Debug, Clone)]
pub struct FnApp {
    pub fun: Box<Expr>,
    pub args: Vec<Expr>,
}

impl FnApp {
    pub fn new(fun: Box<Expr>, args: Vec<Expr>) -> Self {
        Self { fun, args }
    }
}

impl Display for FnApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({} {})",
            self.fun,
            self.args
                .iter()
                .map(|arg| arg.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}

#[derive(Debug, Clone)]
pub enum IntrinsicFn {
    Add,
}

#[derive(Debug, Clone)]
pub struct FnDef {
    pub intrinsic: Option<IntrinsicFn>,
    pub params: Vec<Parameter>,
    pub body: Box<Expr>,
}

impl FnDef {
    pub fn new(params: Vec<Parameter>, body: Box<Expr>) -> Self {
        Self {
            intrinsic: None,
            params,
            body,
        }
    }

    pub fn new_intrinsic(intrinsic: IntrinsicFn, params: Vec<Parameter>, body: Box<Expr>) -> Self {
        Self {
            intrinsic: Some(intrinsic),
            params,
            body,
        }
    }
}

impl Display for FnDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(fn({}))",
            self.params
                .iter()
                .map(|param| param.to_string())
                .collect::<Vec<_>>()
                .join(" "),
        )
    }
}

#[derive(Debug, Clone)]
pub struct Let {
    pub name: String,
    pub typ: Option<Type>,
    pub value: Box<Expr>,
}

impl Let {
    pub fn new(name: String, typ: Option<Type>, value: Box<Expr>) -> Self {
        Self { name, typ, value }
    }
}

impl Display for Let {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "let {}: {:?} = {};", self.name, self.typ, self.value)
    }
}

pub type Value = i64;

pub fn from_expr(expr: &Expr) -> Result<Value> {
    match expr {
        Expr::Literal(v) => Ok(*v),
        _ => Err(anyhow::anyhow!("{} is not value", expr)),
    }
}

#[derive(Debug, Clone)]
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
