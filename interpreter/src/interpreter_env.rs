use anyhow::Result;
use std::{collections::HashMap, fmt::Display};
use structural_typesystem::{
    issuer::{new_variable, Issuer},
    type_env::{default_env, TypeEnv},
    types::{Id, Type},
};

use crate::ast::{Expr, FnDef, Parameter, Value};

#[derive(Debug, Clone)]
pub struct InterpreterEnv {
    pub alloc: Vec<Type>,
    pub type_env: TypeEnv,
    pub variables: HashMap<String, (Id, Value)>,
    pub functions: HashMap<String, FnDef>,
}

impl InterpreterEnv {
    fn intrinsic_fn(alloc: &mut Vec<Type>) -> Result<HashMap<String, FnDef>> {
        let int = Type::from(alloc, "int")?;
        let a = new_variable(alloc);

        let mut fns: HashMap<String, FnDef> = HashMap::new();
        fns.insert(
            "+".to_string(),
            FnDef::new_intrinsic(
                crate::ast::IntrinsicFn::Add,
                vec![
                    Parameter::new("left".to_string(), int.id()),
                    Parameter::new("right".to_string(), int.id()),
                ],
                Box::new(Expr::Literal(Value::Int(0))),
            ),
        );
        fns.insert(
            "=".to_string(),
            FnDef::new_intrinsic(
                crate::ast::IntrinsicFn::Eq,
                vec![
                    Parameter::new("left".to_string(), a),
                    Parameter::new("right".to_string(), a),
                ],
                Box::new(Expr::Literal(Value::Int(0))),
            ),
        );
        fns.insert(
            "zero?".to_string(),
            FnDef::new_intrinsic(
                crate::ast::IntrinsicFn::IsZero,
                vec![Parameter::new("value".to_string(), int.id())],
                Box::new(Expr::Literal(Value::Int(0))),
            ),
        );
        Ok(fns)
    }

    pub fn new() -> Self {
        let (mut alloc, type_env) = default_env();

        let functions = Self::intrinsic_fn(&mut alloc).unwrap();
        Self {
            alloc,
            type_env,
            variables: HashMap::new(),
            functions,
        }
    }

    pub fn new_var(&mut self, name: String, typ_id: Id, val: Value) {
        self.type_env.0.insert(name.clone(), typ_id);
        self.variables.insert(name, (typ_id, val));
    }
}

impl Display for InterpreterEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[env]")?;
        writeln!(f, "type_env:")?;
        for (k, v) in &self.type_env.0 {
            writeln!(f, "{} = #{}", k, v)?;
        }
        writeln!(f, "alloc:")?;
        for typ in &self.alloc {
            writeln!(
                f,
                "#{} = {}",
                typ.id(),
                typ.as_string(&self.alloc, &mut Issuer::new('a'))
            )?;
        }

        writeln!(f, "variables:")?;
        for (name, (typ_id, v)) in &self.variables {
            writeln!(f, "\t{}: #{} = {}\n", name, typ_id, v)?;
        }

        writeln!(f, "functions:")?;
        for (name, fn_def) in &self.functions {
            writeln!(f, "\t{}: {}", name, fn_def)?;
        }
        Ok(())
    }
}
