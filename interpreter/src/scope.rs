use anyhow::{anyhow, Result};
use ast::ast::Expr;
use ast::ast::{FnDef, Parameter, Value};
use std::collections::HashMap;
use std::fmt::Display;
use structural_typesystem::type_env::TypeEnv;
use structural_typesystem::types::Id;
use symbolic_expressions::parser::parse_str;

#[derive(Debug, Clone)]
pub struct Scope {
    pub id: usize,
    pub variables: HashMap<String, (Id, Expr)>,
    pub parent: Option<usize>,
}

pub fn root_scope(type_env: &mut TypeEnv) -> Result<Scope> {
    let not_id = type_env.new_type(&parse_str("(-> bool bool)")?)?;
    let not_impl = Expr::FnDef(FnDef::new(
        Parameter::new("v".to_string(), Some(parse_str("bool")?)),
        Box::new(Expr::Literal(Value::Nil)),
    ));
    let id_id = type_env.new_type(&parse_str("(-> a a)")?)?;
    let id_impl = Expr::FnDef(FnDef::new(
        Parameter::new("v".to_string(), Some(parse_str("a")?)),
        Box::new(Expr::Variable("v".to_string())),
    ));
    let plus_id = type_env.new_type(&parse_str("(-> int (-> int int))")?)?;
    let plus_impl = Expr::FnDef(FnDef::new(
        Parameter::new("a".to_string(), Some(parse_str("int")?)),
        Box::new(Expr::FnDef(FnDef::new(
            Parameter::new("b".to_string(), Some(parse_str("int")?)),
            Box::new(Expr::Literal(Value::Nil)),
        ))),
    ));
    let builtin_variables = HashMap::from_iter(vec![
        ("not".to_string(), (not_id, not_impl)),
        ("id".to_string(), (id_id, id_impl)),
        ("+".to_string(), (plus_id, plus_impl)),
    ]);
    Ok(Scope {
        id: 0,
        variables: builtin_variables,
        parent: None,
    })
}

impl Scope {
    pub fn insert(&mut self, name: &str, ty_id: Id, expr: Expr) {
        self.variables.insert(name.to_string(), (ty_id, expr));
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut (Id, Expr)> {
        self.variables
            .get_mut(name)
            .ok_or(anyhow!("variable {} not found", name))
    }

    pub fn get(&self, name: &str) -> Result<&(Id, Expr)> {
        self.variables
            .get(name)
            .ok_or(anyhow!("variable {} not found", name))
    }
}

impl Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Scope#{} parent={:?}", self.id, self.parent)?;
        for (name, (ty_id, _expr)) in &self.variables {
            writeln!(f, "{} :: {}", name, ty_id)?;
        }
        Ok(())
    }
}
