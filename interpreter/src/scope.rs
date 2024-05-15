use anyhow::{anyhow, Result};
use ast::ast::Expr;
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Scope {
    pub id: usize,
    pub variables: HashMap<String, Expr>,
    pub parent: Option<usize>,
}

impl Default for Scope {
    fn default() -> Self {
        Scope {
            id: 0,
            variables: HashMap::new(),
            parent: None,
        }
    }
}

impl Scope {
    pub fn insert(&mut self, name: &str, expr: Expr) {
        self.variables.insert(name.to_string(), expr);
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut Expr> {
        self.variables
            .get_mut(name)
            .ok_or(anyhow!("variable {} not found", name))
    }

    pub fn get(&self, name: &str) -> Result<&Expr> {
        self.variables
            .get(name)
            .ok_or(anyhow!("variable {} not found", name))
    }
}

impl Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Scope#{} parent={:?}", self.id, self.parent)?;
        for (name, expr) in &self.variables {
            writeln!(f, "{} = {}", name, expr)?;
        }
        Ok(())
    }
}
