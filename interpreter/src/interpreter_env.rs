use crate::builtin::main_context;
use anyhow::{anyhow, Result};
use ast::ast::Expr;
use std::{collections::HashMap, fmt::Display};
use structural_typesystem::{type_env::TypeEnv, types::Id};

#[derive(Debug, Clone)]
pub struct Context {
    pub variables: HashMap<String, Expr>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (_name, expr) in &self.variables {
            writeln!(f, "- {}", expr)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct InterpreterEnv {
    pub current_context: String,
    // TODO: type_env per each context
    pub type_env: TypeEnv,
    pub contexts: HashMap<String, Context>,
}

impl Default for InterpreterEnv {
    fn default() -> Self {
        let mut global_type_env = TypeEnv::default();
        let main_context = main_context(&mut global_type_env).unwrap();
        Self {
            current_context: "main".to_string(),
            type_env: global_type_env,
            contexts: HashMap::from_iter(vec![("main".to_string(), main_context)]),
        }
    }
}

impl InterpreterEnv {
    pub fn switch_context(&mut self, name: &str) -> &mut Context {
        self.current_context = name.to_string();
        self.contexts
            .entry(name.to_string())
            .or_insert_with(Context::new)
    }

    pub fn new_var(&mut self, name: &str, expr: Expr, typ: Id) {
        self.type_env.add(name, typ);
        let context = self.switch_context(name);
        context.variables.insert(name.to_string(), expr);
    }

    pub fn get_variable(&self, name: &str) -> Result<Expr> {
        let ctx = self
            .contexts
            .get(&self.current_context)
            .ok_or(anyhow!("{} not found", name))?;
        if let Some(e @ Expr::Variable(_)) = ctx.variables.get(name) {
            Ok(e.clone())
        } else {
            Err(anyhow!("{} not found (env={})", name, self.current_context))
        }
    }
}

impl Display for InterpreterEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "type_env: {}", self.type_env)?;
        for (name, context) in self.contexts.iter() {
            writeln!(f, "{} {}", name, context)?;
        }
        Ok(())
    }
}
