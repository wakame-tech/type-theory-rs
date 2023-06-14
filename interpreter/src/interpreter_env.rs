use anyhow::Result;
use ast::ast::{Expr, FnDef, Parameter, Value};
use std::{
    collections::HashMap,
    fmt::{Display, Write},
};
use structural_typesystem::{
    builtin_types::register_builtin_types, type_alloc::TypeAlloc, type_env::TypeEnv,
};
use symbolic_expressions::Sexp;

#[derive(Debug, Clone)]
pub struct InterpreterEnv {
    pub alloc: TypeAlloc,
    pub type_env: TypeEnv,
    pub variables: HashMap<String, Expr>,
}

impl InterpreterEnv {
    pub fn new() -> Result<Self> {
        let mut alloc = TypeAlloc::new();
        let mut type_env = TypeEnv::new();
        register_builtin_types(&mut type_env, &mut alloc)?;
        let mut env = Self {
            alloc,
            type_env,
            variables: HashMap::new(),
        };
        register_builtin_vars(&mut env)?;
        Ok(env)
    }
}

fn register_builtin_vars(env: &mut InterpreterEnv) -> Result<()> {
    let int = env.alloc.from("int")?;
    let int_int = env.alloc.new_function(int, int);
    let int_int_int = env.alloc.new_function(int_int, int);
    env.type_env.add("+", int_int_int);
    env.type_env.add("a", int);
    env.type_env.add("b", int);

    let add_fn = FnDef::new(
        &mut env.alloc,
        vec![
            Parameter::new("a".to_string(), int),
            Parameter::new("b".to_string(), int),
        ],
        Box::new(Expr::Literal(Value {
            raw: Sexp::Empty,
            type_id: int,
        })),
    );
    env.new_var("+".to_string(), Expr::FnDef(add_fn));

    let a = env.alloc.new_variable();
    let a_a = env.alloc.new_function(a, a);
    env.type_env.add("v", a);
    env.type_env.add("id", a_a);

    let id_fn = FnDef::new(
        &mut env.alloc,
        vec![Parameter::new("v".to_string(), a)],
        Box::new(Expr::Literal(Value {
            raw: Sexp::Empty,
            type_id: int,
        })),
    );
    env.new_var("id".to_string(), Expr::FnDef(id_fn));

    Ok(())
}

impl InterpreterEnv {
    pub fn new_var(&mut self, name: String, expr: Expr) {
        self.type_env.add(&name, expr.type_id());
        self.variables.insert(name, expr);
    }

    pub fn debug(&self, expr: &Expr) -> Result<String> {
        match expr {
            Expr::Literal(lit) => {
                let typ = self.alloc.as_string(lit.type_id, &mut Default::default())?;
                Ok(format!("{}: {}", lit.raw, typ))
            }
            Expr::Variable(var) => {
                let var = self
                    .variables
                    .get(var)
                    .ok_or(anyhow::anyhow!("not found"))?;
                let typ = self
                    .alloc
                    .as_string(var.type_id(), &mut Default::default())?;
                Ok(format!("{}: {}", var, typ))
            }
            Expr::Let(_) => todo!(),
            Expr::FnApp(app) => {
                let params = app
                    .1
                    .iter()
                    .map(|expr| {
                        self.alloc
                            .as_string(expr.type_id(), &mut Default::default())
                            .map(|ty| format!("{}: {}", self.debug(expr).unwrap(), ty))
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(format!("({})", params.join(" ")))
            }
            Expr::FnDef(def) => {
                let typ = self.alloc.as_string(def.type_id, &mut Default::default())?;
                Ok(format!("{}: {}", def.body, typ))
            }
        }
    }
}

impl Display for InterpreterEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[env]")?;
        writeln!(f, "type_env:")?;
        for (k, v) in &self.type_env.id_map {
            writeln!(f, "{} = #{}", k, v)?;
        }
        writeln!(f, "variables:")?;
        for (_name, expr) in &self.variables {
            writeln!(f, "{}", self.debug(expr).unwrap())?;
        }
        Ok(())
    }
}
