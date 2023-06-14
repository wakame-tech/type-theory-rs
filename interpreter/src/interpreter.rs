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
        let typ = if let Some(typ) = &self.typ {
            env.type_env.alloc.from_sexp(typ)?
        } else {
            todo!();
        };
        env.new_var(&self.name, expr, typ);
        Ok(Expr::Variable(self.name.clone()))
    }
}

impl Eval for FnApp {
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr> {
        let param = self.1.eval(env)?;
        let f = match self.0.eval(env)? {
            // builtin function
            Expr::Variable(plus) if plus == "+" => {
                todo!()
            }
            Expr::Variable(name) => {
                if let Expr::FnDef(fn_def) = dbg!(env.get_variable(&name)?) {
                    Ok(fn_def)
                } else {
                    Err(anyhow!("{} is cannot apply", name))
                }
            }
            Expr::FnDef(def) => Ok(def),
            expr => Err(anyhow!("{} cannot apply", expr)),
        }?;
        let current_context = env.current_context.clone();
        let context = env.switch_context(f.to_string().as_str());
        context.variables.insert(f.param.name, param);
        let ret = f.body.eval(env);
        env.switch_context(&current_context);
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
            Expr::Variable(var) => env.get_variable(var),
        }
    }
}

impl Eval for Program {
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr> {
        self.0.eval(env)
    }
}
