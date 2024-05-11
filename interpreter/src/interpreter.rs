use crate::traits::InferType;
use crate::{interpreter_env::InterpreterEnv, traits::Eval};
use anyhow::{anyhow, Ok, Result};
use ast::{
    ast::{Expr, FnApp, FnDef, Let, Program, Value},
    into_ast::into_ast,
};

impl Eval for FnDef {
    fn eval(&self, _env: &mut InterpreterEnv) -> Result<Expr> {
        log::debug!("FnDef::eval {}", self);
        Ok(Expr::FnDef(self.clone()))
    }
}

impl Eval for Let {
    /// (let a int 1)
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr> {
        log::debug!("Let::eval {}", self);
        let value = self.value.eval(env)?;
        let ty_id = if let Some(ty) = &self.typ {
            env.type_env.get(ty)?
        } else {
            self.value.infer_type(env, &mut Default::default())?
        };
        env.current_mut()
            .insert(&self.name, ty_id, *self.value.clone());
        Ok(value)
    }
}

impl Eval for FnApp {
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr> {
        log::debug!("FnApp::eval {}", self);
        let (f, arg) = (self.0.eval(env)?, self.1.eval(env)?);
        let f = match f {
            Expr::Variable(name) => {
                if let (_, Expr::FnDef(fn_def)) = env.current().get(&name)?.clone() {
                    Ok(fn_def)
                } else {
                    Err(anyhow!("{} is cannot apply", name))
                }
            }
            Expr::FnDef(def) => Ok(def),
            expr => Err(anyhow!("{} cannot apply", expr)),
        }?;

        let param_ty = if let Some(arg_ty) = &f.arg.typ {
            env.type_env.get(arg_ty)?
        } else {
            self.1.infer_type(env, &mut Default::default())?
        };

        let scope = env.current().clone();
        let scope = env.new_scope(scope);
        scope.insert(&f.arg.name, param_ty, arg.clone());
        log::debug!("@#{} bind {} = {}", scope.id, f.arg.name, arg);
        let res = f.body.eval(env)?;
        log::debug!("FnApp::eval {} {} = {}", f, arg, res);
        env.pop_scope();
        Ok(res)
    }
}

impl Eval for Expr {
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr> {
        let ret = match self {
            Expr::FnDef(fndef) => fndef.eval(env),
            Expr::Let(r#let) => r#let.eval(env),
            Expr::FnApp(fnapp) => fnapp.eval(env),
            Expr::Literal(lit) => Ok(Expr::Literal(lit.clone())),
            Expr::Variable(var) => Ok(env.current().get(var)?.1.clone()),
        };
        log::debug!("eval scope={} {}", env.current(), self);
        ret
    }
}

impl Eval for Program {
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr> {
        let exprs = self
            .0
            .iter()
            .map(|expr| expr.eval(env))
            .collect::<Result<Vec<_>>>()?;
        Ok(exprs.last().unwrap().clone())
    }
}
