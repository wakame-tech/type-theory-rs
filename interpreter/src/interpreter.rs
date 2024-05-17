use crate::interpreter_env::InterpreterEnv;
use anyhow::{anyhow, Ok, Result};
use ast::ast::{Expr, FnApp, FnDef, Let, Program};

pub trait Eval {
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr>;
}

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
        env.current_mut().insert(&self.name, *self.value.clone());
        Ok(value)
    }
}

impl Eval for FnApp {
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr> {
        log::debug!("FnApp::eval {}", self);
        let f = self.0.eval(env)?;
        let args = self
            .1
            .iter()
            .map(|arg| arg.eval(env))
            .collect::<Result<Vec<_>>>()?;
        let f = match f {
            Expr::Variable(name) => {
                match name.as_str() {
                    "+" => todo!(),
                    "-" => todo!(),
                    _ => {}
                }
                if let Expr::FnDef(fn_def) = env.current().get(&name)?.clone() {
                    Ok(fn_def)
                } else {
                    Err(anyhow!("{} is cannot apply", name))
                }
            }
            Expr::FnDef(def) => Ok(def),
            expr => Err(anyhow!("{} cannot apply", expr)),
        }?;

        let scope = env.current().clone();
        let scope = env.new_scope(scope);
        for (param, arg) in f.args.iter().zip(args.iter()) {
            scope.insert(&param.name, arg.clone());
            log::debug!("@#{} bind {} = {}", scope.id, &param.name, arg);
        }
        let res = f.body.eval(env)?;
        log::debug!("FnApp::eval {} {:?} = {}", f, args, res);
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
            Expr::Variable(var) => Ok(env.current().get(var)?.clone()),
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
