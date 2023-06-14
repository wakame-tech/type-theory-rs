use crate::{interpreter_env::InterpreterEnv, traits::Eval};
use anyhow::{anyhow, Ok, Result};
use ast::ast::{Expr, FnApp, FnDef, Let, Program};

impl Eval for FnDef {
    fn eval(&self, _env: &mut InterpreterEnv) -> Result<Expr> {
        Ok(Expr::FnDef(self.clone()))
    }
}

impl Eval for Let {
    // (let a int 1)
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr> {
        let expr = self.value.eval(env)?;
        env.new_var(self.name.clone(), expr);
        Ok(Expr::Variable(self.name.clone()))
    }
}

impl Eval for FnApp {
    /// (f g 1) => apply(f, apply(g, 1))
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr> {
        self.1
            .clone()
            .into_iter()
            .map(Ok)
            .rev()
            .reduce(|f, v| apply(env, &f?, &v?))
            .unwrap()
    }
}

fn apply(env: &mut InterpreterEnv, f: &Expr, param: &Expr) -> Result<Expr> {
    let param = param.eval(env)?;
    let fn_expr = match f.eval(env)? {
        Expr::Variable(name) => env
            .variables
            .get(&name)
            .cloned()
            .ok_or(anyhow!("variable {} not found", name)),
        def @ Expr::FnDef(_) => Ok(def),
        _ => Err(anyhow!("")),
    }?;

    let Expr::FnDef(fn_def) = fn_expr else {
        return Err(anyhow!("{} cannot apply", f))
    };

    let mut env = env.clone();
    for (param, val) in fn_def.params.iter().zip(vec![param].iter()) {
        env.new_var(param.name.clone(), val.clone());
    }
    fn_def.body.eval(&mut env)
}

impl Eval for Expr {
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr> {
        match self {
            Expr::FnDef(fndef) => fndef.eval(env),
            Expr::Let(r#let) => r#let.eval(env),
            Expr::FnApp(fnapp) => fnapp.eval(env),
            Expr::Literal(lit) => Ok(Expr::Literal(lit.clone())),
            Expr::Variable(var) => {
                if let Some(lit @ Expr::Literal(_)) = env.variables.get(var) {
                    Ok(lit.clone())
                } else {
                    Err(anyhow::anyhow!("variable {} not found", var))
                }
            }
        }
    }
}

impl Eval for Program {
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr> {
        self.0.eval(env)
    }
}
