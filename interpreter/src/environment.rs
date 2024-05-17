use anyhow::{anyhow, Result};
use ast::ast::Expr;
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Environment {
    pub variables: HashMap<String, Expr>,
    pub parent: Option<Box<Environment>>,
}

impl Environment {
    pub fn new(parent: Option<Box<Environment>>) -> Self {
        Self {
            variables: HashMap::new(),
            parent,
        }
    }

    pub fn insert(&mut self, name: &str, expr: Expr) {
        self.variables.insert(name.to_string(), expr);
    }

    pub fn get(&self, name: &str) -> Result<&Expr> {
        self.variables
            .get(name)
            .ok_or(anyhow!("variable {} not found", name))
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (name, expr) in &self.variables {
            writeln!(f, "  {} = {}", name, expr)?;
        }
        if let Some(parent) = &self.parent {
            writeln!(f, "parent:\n{}", parent)?;
        }
        Ok(())
    }
}
