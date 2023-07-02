use crate::{interpreter_env::InterpreterEnv, traits::TypeCheck};
use anyhow::Result;
use ast::ast::{Expr, FnApp, FnDef, Let, MacroApp, Program};
use structural_typesystem::{
    subtyping::is_subtype,
    types::{Id, Type},
};
use symbolic_expressions::{parser::parse_str, Sexp};

impl TypeCheck for FnDef {
    fn type_check(&self, env: &mut InterpreterEnv) -> Result<Id> {
        let param_typ = env.type_env.new_type(&self.param.typ)?;
        env.new_var(
            &self.param.name,
            Expr::Variable(self.param.name.clone()),
            param_typ,
        );

        let body_typ = self.body.type_check(env)?;
        let f_typ = env.type_env.alloc.new_function(param_typ, body_typ);
        Ok(f_typ)
    }
}

impl TypeCheck for Let {
    fn type_check(&self, env: &mut InterpreterEnv) -> Result<Id> {
        let value_typ = self.value.type_check(env)?;

        // TODO: type inference
        if let Some(typ) = &self.typ {
            let ty_id = env.type_env.new_type(typ)?;
            is_subtype(&mut env.type_env, value_typ, ty_id)?;
        } else {
            todo!()
        }
        env.new_var(&self.name, Expr::Variable(self.name.clone()), value_typ);
        Ok(value_typ)
    }
}

impl TypeCheck for FnApp {
    /// f :: a -> b
    /// v :: a
    fn type_check(&self, env: &mut InterpreterEnv) -> Result<Id> {
        let f_type = self.0.type_check(env)?;
        let Type::Operator { name, types, .. } = env.type_env.alloc.from_id(f_type)? else {
            return Err(anyhow::anyhow!("{} is not appliable type", self.0))
        };
        anyhow::ensure!(name == "->");
        let (arg_ty, ret_ty) = (types[0], types[1]);
        let param_ty = self.1.type_check(env)?;
        is_subtype(&mut env.type_env, param_ty, arg_ty)?;
        Ok(ret_ty)
    }
}

impl TypeCheck for MacroApp {
    fn type_check(&self, env: &mut InterpreterEnv) -> Result<Id> {
        let ret_ty = match self.0.list()?[0].string()?.as_str() {
            "add!" => "int",
            "not!" => "bool",
            _ => panic!(),
        };
        env.type_env.get(&parse_str(ret_ty)?)
    }
}

impl TypeCheck for Expr {
    fn type_check(&self, env: &mut InterpreterEnv) -> Result<Id> {
        let ret = match self {
            Expr::Literal(value) => match &value.raw {
                Sexp::String(lit) if lit.parse::<usize>().is_ok() => {
                    env.type_env.get(&parse_str("int")?)
                }
                Sexp::String(lit) if lit == "true" || lit == "false" => {
                    env.type_env.get(&parse_str("bool")?)
                }
                sexp => Err(anyhow::anyhow!("unknown literal {}", sexp)),
            },
            Expr::Variable(name) => Ok(env.get_variable(name)?.0),
            Expr::Let(lt) => lt.type_check(env),
            Expr::FnApp(app) => app.type_check(env),
            Expr::FnDef(fn_def) => fn_def.type_check(env),
            Expr::MacroApp(macro_app) => macro_app.type_check(env),
        }?;
        log::debug!(
            "type_check {} :: {}",
            self,
            env.type_env.alloc.as_sexp(ret, &mut Default::default())?
        );
        Ok(ret)
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
