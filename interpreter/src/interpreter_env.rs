use anyhow::Result;
use ast::ast::{Expr, Value};
use std::{collections::HashMap, fmt::Display};
use structural_typesystem::{type_alloc::TypeAlloc, type_env::TypeEnv};

#[derive(Debug, Clone)]
pub struct InterpreterEnv {
    pub alloc: TypeAlloc,
    pub type_env: TypeEnv,
    pub variables: HashMap<String, Expr>,
}

impl Default for InterpreterEnv {
    fn default() -> Self {
        let mut env = InterpreterEnv {
            alloc: Default::default(),
            type_env: Default::default(),
            variables: HashMap::new(),
        };
        register_intrinsic_fns(&mut env).unwrap();
        env
    }
}

fn register_intrinsic_fns(env: &mut InterpreterEnv) -> Result<()> {
    return Ok(());

    // let int = env.alloc.from("int")?;
    // let a = env.alloc.new_variable();

    // let mut fns: HashMap<String, FnDef> = HashMap::new();
    // let sexp = parse_str("(lam (left : int) lam (right : int) (+ left right))")?;
    // let ast = into_ast(alloc, &sexp).unwrap();
    // fns.insert(
    //     "+".to_string(),
    //     FnDef::new(
    //         IntrinsicFn::Add,
    //         vec![
    //             Parameter::new("left".to_string(), int.id()),
    //             Parameter::new("right".to_string(), int.id()),
    //         ],
    //         Box::new(Expr::Literal(Value::Int(0))),
    //     ),
    // );
    // fns.insert(
    //     "=".to_string(),
    //     FnDef::new_intrinsic(
    //         IntrinsicFn::Eq,
    //         vec![
    //             Parameter::new("left".to_string(), a),
    //             Parameter::new("right".to_string(), a),
    //         ],
    //         Box::new(Expr::Literal(Value::Int(0))),
    //     ),
    // );
    // fns.insert(
    //     "zero?".to_string(),
    //     FnDef::new_intrinsic(
    //         IntrinsicFn::IsZero,
    //         vec![Parameter::new("value".to_string(), int.id())],
    //         Box::new(Expr::Literal(Value::Int(0))),
    //     ),
    // );
    // Ok(())
}

impl InterpreterEnv {
    pub fn new_var(&mut self, name: String, expr: Expr) {
        self.type_env.add(&name, expr.type_id());
        self.variables.insert(name, expr);
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
        for (name, expr) in &self.variables {
            writeln!(f, "\t{}: #{} = {}\n", name, expr.type_id(), expr)?;
        }
        Ok(())
    }
}
