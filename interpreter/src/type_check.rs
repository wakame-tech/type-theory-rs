use crate::{interpreter_env::InterpreterEnv, traits::TypeCheck};
use anyhow::Result;
use ast::ast::{Expr, FnApp, FnDef, Let, MacroApp, Program};
use structural_typesystem::{subtyping::is_subtype, types::Id};
use symbolic_expressions::{parser::parse_str, Sexp};

impl TypeCheck for FnDef {
    fn type_check(&self, env: &mut InterpreterEnv) -> Result<Id> {
        let param_typ = env.type_env.alloc.from_sexp(&self.param.typ)?;
        let body_typ = self.body.type_check(env)?;
        let f_typ = env.type_env.alloc.new_function(param_typ, body_typ);
        Ok(f_typ)
    }
}

impl TypeCheck for Let {
    fn type_check(&self, env: &mut InterpreterEnv) -> Result<Id> {
        let value_typ = self.value.type_check(env)?;
        if let Some(typ) = &self.typ {
            let typ = env.type_env.alloc.from_sexp(typ)?;
            is_subtype(&mut env.type_env, value_typ, typ)?;
        }
        env.new_var(&self.name, Expr::Variable(self.name.clone()), value_typ);
        Ok(value_typ)
    }
}

impl TypeCheck for FnApp {
    /// f :: a -> b
    /// v :: a
    fn type_check(&self, env: &mut InterpreterEnv) -> Result<Id> {
        let _f_type = self.0.type_check(env)?;
        let v_type = self.1.type_check(env)?;
        Ok(v_type)
    }
}

impl TypeCheck for MacroApp {
    fn type_check(&self, env: &mut InterpreterEnv) -> Result<Id> {
        env.type_env.get(&parse_str("any")?)
    }
}

impl TypeCheck for Expr {
    fn type_check(&self, env: &mut InterpreterEnv) -> Result<Id> {
        match self {
            Expr::Literal(value) => match &value.raw {
                Sexp::String(lit) if lit.parse::<usize>().is_ok() => {
                    env.type_env.get(&parse_str("int")?)
                }
                _ => panic!(),
            },
            Expr::Variable(name) => Ok(env.get_variable(name)?.0),
            Expr::Let(lt) => lt.type_check(env),
            Expr::FnApp(app) => app.type_check(env),
            Expr::FnDef(fn_def) => fn_def.type_check(env),
            Expr::MacroApp(macro_app) => macro_app.type_check(env),
        }
    }
}

impl TypeCheck for Program {
    fn type_check(&self, env: &mut InterpreterEnv) -> Result<Id> {
        let id = *self
            .0
            .iter()
            .map(|expr| expr.type_check(env))
            .collect::<Result<Vec<_>>>()?
            .last()
            .unwrap();
        Ok(id)
    }
}
