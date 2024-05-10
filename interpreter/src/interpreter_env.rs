use crate::builtin::main_context;
use anyhow::{anyhow, Result};
use ast::ast::Expr;
use petgraph::{prelude::NodeIndex, Graph};
use std::{collections::HashMap, fmt::Display};
use structural_typesystem::{type_env::TypeEnv, types::Id};

#[derive(Debug, Clone)]
pub struct Context {
    pub name: String,
    variables: HashMap<String, (Id, Expr)>,
}

impl Context {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            variables: HashMap::new(),
        }
    }

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

#[derive(Debug, Clone)]
pub struct InterpreterEnv {
    // TODO: type_env per each context
    pub type_env: TypeEnv,
    pub current_context: NodeIndex,
    pub context_map: HashMap<String, NodeIndex>,
    pub context_tree: Graph<Context, ()>,
}

impl Default for InterpreterEnv {
    fn default() -> Self {
        let mut global_type_env = TypeEnv::default();
        let main_context = main_context(&mut global_type_env).unwrap();
        let mut context_tree = Graph::new();
        let ni = context_tree.add_node(main_context);

        Self {
            current_context: ni,
            type_env: global_type_env,
            context_map: HashMap::from_iter(vec![("main".to_string(), ni)]),
            context_tree,
        }
    }
}

impl InterpreterEnv {
    pub fn context(&self) -> &Context {
        &self.context_tree[self.current_context]
    }

    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context_tree[self.current_context]
    }

    pub fn new_context(&mut self, name: &str) -> NodeIndex {
        let ctx = Context::new(name);
        let ni = self.context_tree.add_node(ctx);
        self.context_map.insert(name.to_string(), ni);
        ni
    }

    pub fn move_context(&mut self, from: &str, to: &str) {
        let (from_ni, to_ni) = (self.context_map[from], self.context_map[to]);
        let (old_ctx_vars, new_ctx) = (
            self.context_tree[from_ni].variables.clone(),
            &mut self.context_tree[to_ni],
        );
        new_ctx.variables = old_ctx_vars;
        self.context_tree.remove_node(from_ni);
        self.context_map.remove(from);
    }

    /// set current context as parent
    pub fn switch_context(&mut self, name: &str) -> &mut Context {
        if let Some(ni) = self.context_map.get(name) {
            self.current_context = *ni;
            log::debug!(
                "switch_ctx #{} -> #{}",
                self.current_context.index(),
                ni.index()
            );
            &mut self.context_tree[*ni]
        } else {
            // add
            let ni = self.new_context(name);
            let parent = self.current_context;
            println!("new ctx '{}'(#{}) created", name, ni.index());

            // link
            self.context_tree.add_edge(parent, ni, ());
            println!("link #{} -> #{}", parent.index(), ni.index());

            // switch
            self.current_context = ni;

            &mut self.context_tree[ni]
        }
    }

    pub fn new_var(&mut self, name: &str, expr: Expr, ty_id: Id) {
        let ctx = self.context_mut();
        ctx.insert(name, ty_id, expr);
    }

    fn find_context(&self, name: &str) -> Result<NodeIndex> {
        let mut ni = self.current_context;
        let mut ctx_trace = vec![];
        loop {
            let ctx = &self.context_tree[ni];
            ctx_trace.push(ctx.name.clone());
            if ctx.variables.contains_key(name) {
                return Ok(ni);
            }
            if let Some(parent_ni) = self
                .context_tree
                .neighbors_directed(ni, petgraph::Direction::Incoming)
                .collect::<Vec<_>>()
                .first()
            {
                ni = *parent_ni;
            } else {
                return Err(anyhow!(
                    "variable {} not found\ntrace: {:?}",
                    name,
                    ctx_trace
                ));
            }
        }
    }

    pub fn get_variable(&self, name: &str) -> Result<&(Id, Expr)> {
        let ni = self.find_context(name)?;
        self.context_tree[ni].get(name)
    }

    pub fn get_variable_mut(&mut self, name: &str) -> Result<&mut (Id, Expr)> {
        let ni = self.find_context(name)?;
        self.context_tree[ni].get_mut(name)
    }
}

impl Display for InterpreterEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\n{}", self.type_env)?;
        for context in self.context_tree.node_weights() {
            writeln!(f, "[{}]", context.name)?;
            for (name, (ty_id, _expr)) in &context.variables {
                writeln!(
                    f,
                    "#{}: {} :: {}",
                    ty_id,
                    name,
                    self.type_env.get_by_id(ty_id).unwrap()
                )?;
            }
        }
        Ok(())
    }
}
