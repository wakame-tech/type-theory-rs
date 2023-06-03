use anyhow::{anyhow, Result};
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};
use structural_typesystem::{type_alloc::TypeAlloc, types::Id};

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
    pub type_id: Id,
}

impl FnApp {
    pub fn new(alloc: &mut TypeAlloc, fun: Box<Expr>, args: Vec<Expr>) -> Self {
        // TODO
        let type_id = alloc.from("any").unwrap();
        Self { fun, args, type_id }
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
pub struct FnDef {
    pub params: Vec<Parameter>,
    pub body: Box<Expr>,
    pub type_id: Id,
}

impl FnDef {
    pub fn new(alloc: &mut TypeAlloc, params: Vec<Parameter>, body: Box<Expr>) -> Self {
        let type_ids: Vec<_> = params.iter().map(|p| p.typ_id).collect();
        let type_id = type_ids
            .into_iter()
            .rev()
            .fold(body.type_id(), |acc, id| alloc.new_function(id, acc));

        Self {
            params,
            body,
            type_id,
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
    pub type_id: Id,
    pub value: Box<Expr>,
}

impl Let {
    /// if [typ_id] is None, will infer the type from [value]
    pub fn new(name: String, type_id: Id, value: Box<Expr>) -> Self {
        Self {
            name,
            type_id,
            value,
        }
    }
}

impl Display for Let {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "let {}: #{} = {};", self.name, self.type_id, self.value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Value {
    pub raw: String,
    pub type_id: Id,
}

impl Value {
    pub fn as_value<T: FromStr>(&self) -> Result<T> {
        self.raw.parse::<T>().map_err(|_| anyhow!("cannot parse"))
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: #{}", self.raw, self.type_id)
    }
}

pub fn from_expr(expr: &Expr) -> Result<Value> {
    match expr {
        Expr::Literal(v) => Ok(v.clone()),
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

    pub fn type_id(&self) -> Id {
        match self {
            Expr::Literal(lit) => lit.type_id,
            Expr::Variable(v) => 0,
            Expr::Let(lt) => lt.type_id,
            Expr::FnApp(app) => app.type_id,
            Expr::FnDef(def) => def.type_id,
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
