use anyhow::Result;
use std::fmt::{Debug, Display};
use structural_typesystem::{type_alloc::TypeAlloc, types::Id};
use symbolic_expressions::Sexp;

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

#[derive(Debug, Clone, PartialEq)]
pub struct FnApp(pub Id, pub Vec<Expr>);

impl FnApp {
    pub fn new(alloc: &mut TypeAlloc, apps: Vec<Expr>) -> Self {
        // TODO
        let type_id = alloc.from("any").unwrap();
        Self(type_id, apps)
    }
}

impl Display for FnApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({})",
            self.1
                .iter()
                .map(|arg| arg.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
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
            "({}): {} = {}",
            self.params
                .iter()
                .map(|param| param.to_string())
                .collect::<Vec<_>>()
                .join(" "),
            self.type_id,
            self.body,
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    pub raw: Sexp,
    pub type_id: Id,
}

impl Value {
    pub fn new(raw: Sexp, type_id: Id) -> Self {
        Value { raw, type_id }
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

    pub fn type_id(&self) -> Id {
        match self {
            Expr::Literal(lit) => lit.type_id,
            Expr::Variable(v) => 0,
            Expr::Let(lt) => lt.type_id,
            Expr::FnApp(app) => app.0,
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
pub struct Program(pub Expr);
