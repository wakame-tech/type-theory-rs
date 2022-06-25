use anyhow::Result;
use structural_typesystem::types::Type;

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub typ: Option<Type>,
}

impl Variable {
    pub fn new(name: String, typ: Option<Type>) -> Self {
        Self { name, typ }
    }
}

#[derive(Debug)]
pub struct FnApp {
    pub name: String,
    pub args: Vec<Expr>,
}

impl FnApp {
    pub fn new(name: String, args: Vec<Expr>) -> Self {
        Self { name, args }
    }
}

#[derive(Debug)]
pub struct FnDef {
    pub name: String,
    pub params: Vec<Variable>,
    pub body: Box<Expr>,
}

impl FnDef {
    pub fn new(name: String, params: Vec<Variable>, body: Box<Expr>) -> Self {
        Self { name, params, body }
    }
}

#[derive(Debug)]
pub enum Expr {
    Literal(i64),
    Variable(String),
    FnApp(FnApp),
    FnDef(FnDef),
}

impl Expr {
    pub fn literal(self) -> Result<i64> {
        match self {
            Expr::Literal(literal) => Ok(literal),
            _ => Err(anyhow::anyhow!("literal expected")),
        }
    }
}

#[derive(Debug)]
pub struct Program(pub Vec<Expr>);
