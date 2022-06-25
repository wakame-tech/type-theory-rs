use std::fmt::Display;

use anyhow::{anyhow, Result};
use structural_typesystem::types::Id;

use crate::interpreter_env::InterpreterEnv;

pub trait Eval {
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub typ_id: Id,
}

impl Parameter {
    pub fn new(name: String, typ_id: Id) -> Self {
        Self { name, typ_id }
    }
}

impl Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: #{}", self.name, self.typ_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntrinsicFn {
    Add,
    Eq,
    IsZero,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Let {
    pub name: String,
    pub typ_id: Id,
    pub value: Box<Expr>,
}

impl Let {
    pub fn new(name: String, typ_id: Id, value: Box<Expr>) -> Self {
        Self {
            name,
            typ_id,
            value,
        }
    }
}

impl Display for Let {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "let {}: #{} = {};", self.name, self.typ_id, self.value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {
    Nil,
    Int(i64),
    Bool(bool),
}

impl Value {
    pub fn as_int(&self) -> Result<i64> {
        match self {
            Value::Int(i) => Ok(*i),
            _ => Err(anyhow!("Value is not an integer")),
        }
    }

    pub fn as_bool(&self) -> Result<bool> {
        match self {
            Value::Bool(b) => Ok(*b),
            _ => Err(anyhow!("Value is not a boolean")),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Int(i) => write!(f, "{}", i),
            Value::Bool(b) => write!(f, "{}", b),
        }
    }
}

pub fn from_expr(expr: &Expr) -> Result<Value> {
    match expr {
        Expr::Literal(v) => Ok(*v),
        _ => Err(anyhow::anyhow!("{} is not value", expr)),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
