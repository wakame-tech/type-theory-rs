use anyhow::Result;
use std::{collections::HashMap, fmt::Display};
use structural_typesystem::{
    issuer::Issuer,
    type_env::{default_env, TypeEnv},
    types::Type,
};

use crate::ast::{Expr, FnDef, Parameter};

type Value = i64;

#[derive(Debug, Clone)]
pub struct InterpreterEnv {
    pub alloc: Vec<Type>,
    pub type_env: TypeEnv,
    pub variables: HashMap<String, (Type, Value)>,
    pub functions: HashMap<String, FnDef>,
}

impl InterpreterEnv {
    fn intrinsic_fn(alloc: &Vec<Type>) -> Result<HashMap<String, FnDef>> {
        let int = Type::from(alloc, "int")?;

        let mut fns: HashMap<String, FnDef> = HashMap::new();
        fns.insert(
            "+".to_string(),
            FnDef::new_intrinsic(
                crate::ast::IntrinsicFn::Add,
                vec![
                    Parameter::new("left".to_string(), int.clone()),
                    Parameter::new("right".to_string(), int.clone()),
                ],
                Box::new(Expr::Literal(0)),
            ),
        );
        Ok(fns)
    }

    pub fn new() -> Self {
        let (alloc, type_env) = default_env();

        let functions = Self::intrinsic_fn(&alloc).unwrap();
        Self {
            alloc,
            type_env,
            variables: HashMap::new(),
            functions,
        }
    }

    pub fn new_var(&mut self, name: String, typ: Type, val: Value) {
        self.alloc.push(typ.clone());
        self.variables.insert(name, (typ.clone(), val));
    }
}

impl Display for InterpreterEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[env]")?;
        writeln!(f, "variables:")?;
        for (name, (typ, v)) in &self.variables {
            writeln!(
                f,
                "\t{}: {} = {}\n",
                name,
                typ.as_string(&self.alloc, &mut Issuer::new('a')),
                v
            )?;
        }

        writeln!(f, "functions:")?;
        for (name, fn_def) in &self.functions {
            writeln!(f, "\t{}: {}", name, fn_def)?;
        }
        Ok(())
    }
}
