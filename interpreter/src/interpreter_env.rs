use crate::scope::Scope;
use std::{collections::HashMap, fmt::Display};
use structural_typesystem::type_env::TypeEnv;

#[derive(Debug, Clone)]
pub struct InterpreterEnv {
    // TODO: type_env per each context
    pub type_env: TypeEnv,
    current_index: usize,
    scopes: HashMap<usize, Scope>,
}

impl Default for InterpreterEnv {
    fn default() -> Self {
        let global_type_env = TypeEnv::default();
        let scopes = HashMap::new();
        Self {
            current_index: 0,
            type_env: global_type_env,
            scopes,
        }
    }
}

impl InterpreterEnv {
    pub fn current(&self) -> &Scope {
        &self.scopes[&self.current_index]
    }

    pub fn current_mut(&mut self) -> &mut Scope {
        self.scopes.get_mut(&self.current_index).unwrap()
    }

    pub fn new_scope(&mut self, parent: Scope) -> &mut Scope {
        let id = self.scopes.len();
        let ctx = Scope {
            id,
            parent: Some(parent.id),
            variables: parent.variables.clone(),
        };
        self.scopes.insert(id, ctx);
        self.current_index = id;
        self.current_mut()
    }

    pub fn pop_scope(&mut self) {
        if let Some(parent) = self.current().parent {
            self.scopes.remove(&self.current_index);
            self.current_index = parent;
        }
    }
}

impl Display for InterpreterEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\n{}", self.type_env)?;
        for (_, context) in &self.scopes {
            for (name, expr) in &context.variables {
                writeln!(f, "#{} = {}", name, expr)?;
            }
        }
        Ok(())
    }
}
