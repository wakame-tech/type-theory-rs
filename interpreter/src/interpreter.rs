use crate::{eval::Eval, interpreter_env::InterpreterEnv};
use anyhow::{Ok, Result};
use ast::ast::{from_expr, Expr, FnApp, FnDef, Let, Program};
use rand::{distributions::Alphanumeric, Rng};

fn random_name() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect::<String>()
}

impl Eval for FnDef {
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr> {
        let name = random_name();
        env.functions.insert(name.clone(), self.clone());
        Ok(Expr::Variable(name))
    }
}

impl Eval for Let {
    // (let a int 1)
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr> {
        let val = self.value.eval(env)?.literal()?;
        env.new_var(self.name.clone(), val.type_id, val);
        Ok(Expr::Variable(self.name.clone()))
    }
}

impl Eval for FnApp {
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr> {
        let param = self
            .args
            .iter()
            .map(|arg| arg.eval(env).and_then(|arg| from_expr(&arg)))
            .collect::<Result<Vec<_>>>()?;

        let Some(fun) = (match &*self.fun {
            Expr::Variable(fn_name) => env.functions.get(fn_name),
            Expr::FnDef(fndef) => {
                let name = fndef.eval(env)?.name()?;
                env.functions.get(&name)
            }
            _ => return Err(anyhow::anyhow!("{} is not callable", self.fun)),
        }) else {
            return Err(anyhow::anyhow!("{} is not found", self.fun));
        };
        let mut env = env.clone();
        for (param, val) in fun.params.iter().zip(param) {
            env.new_var(param.name.clone(), param.typ_id.clone(), val);
        }
        let ret = fun.body.eval(&mut env);
        println!("{} env\n {}", self.fun, env);
        ret
    }
}

impl Eval for Expr {
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr> {
        match self {
            Expr::FnDef(fndef) => fndef.eval(env),
            Expr::Let(r#let) => r#let.eval(env),
            Expr::FnApp(fnapp) => fnapp.eval(env),
            Expr::Literal(lit) => Ok(Expr::Literal(lit.clone())),
            Expr::Variable(var) => {
                if let Some((_, v)) = env.variables.get(var) {
                    Ok(Expr::Literal(v.clone()))
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
