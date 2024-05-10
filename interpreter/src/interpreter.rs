use crate::{interpreter_env::InterpreterEnv, traits::Eval};
use anyhow::{anyhow, Ok, Result};
use ast::{
    ast::{Expr, FnApp, FnDef, Let, MacroApp, Program, Value},
    into_ast::into_ast,
};
use symbolic_expressions::Sexp;

impl Eval for FnDef {
    fn eval(&self, _env: &mut InterpreterEnv) -> Result<Expr> {
        log::debug!("FnDef::eval {}", self);
        Ok(Expr::FnDef(self.clone()))
    }
}

impl Eval for Let {
    /// (let a int 1)
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr> {
        let value = self.value.eval(env)?;
        // if value has context, move it
        if env.context_map.contains_key(&self.value.to_string()) {
            env.new_context(&self.name);
            env.move_context(&self.value.to_string(), &self.name);
            println!("move ctx {} -> {}", value, self.name);
        }
        let (_, expr) = env.get_variable_mut(&self.name)?;
        *expr = value.clone();
        Ok(value)
    }
}

impl Eval for FnApp {
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr> {
        log::debug!("FnApp::eval {}", self);
        let original_ctx = &env.context().name.to_string();
        // eval param
        let param = self.1.eval(env)?;

        // get fn body
        let f = match self.0.eval(env)? {
            Expr::Variable(name) => {
                if let (_, Expr::FnDef(fn_def)) = env.get_variable(&name)?.clone() {
                    env.switch_context(&name);
                    Ok(fn_def)
                } else {
                    Err(anyhow!("{} is cannot apply", name))
                }
            }
            Expr::FnDef(def) => Ok(def),
            expr => Err(anyhow!("{} cannot apply", expr)),
        }?;

        let ty_id = if let Some(typ) = &f.param.typ {
            env.type_env.get(typ)?
        } else {
            todo!()
        };

        // scope
        let ctx = env.context_mut();
        ctx.insert(&f.param.name, ty_id, param.clone());
        log::debug!(
            "@#{} bind {} = {}",
            env.current_context.index(),
            f.param.name,
            param
        );
        env.switch_context(original_ctx);

        f.body.eval(env)
    }
}

impl Eval for MacroApp {
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr> {
        let (macr, params) = (&self.0.list()?[0].string()?, &self.0.list()?[1..]);
        let values = params
            .iter()
            .map(|expr| into_ast(expr).and_then(|e| e.eval(env)))
            .collect::<Result<Vec<_>>>()?;
        match macr.as_str() {
            "add!" => {
                let (a, b) = (
                    values[0].clone().literal()?.raw.i()?,
                    values[1].clone().literal()?.raw.i()?,
                );
                Ok(Expr::Literal(Value::new(Sexp::String((a + b).to_string()))))
            }
            "not!" => {
                let v = values[0].clone().literal()?.raw.s()? == "true";
                Ok(Expr::Literal(Value::new(Sexp::String((!v).to_string()))))
            }
            _ => Err(anyhow!("macro \"{}\" not found", macr)),
        }
    }
}

impl Eval for Expr {
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr> {
        let ret = match self {
            Expr::FnDef(fndef) => fndef.eval(env),
            Expr::Let(r#let) => r#let.eval(env),
            Expr::FnApp(fnapp) => fnapp.eval(env),
            Expr::Literal(lit) => Ok(Expr::Literal(lit.clone())),
            Expr::Variable(var) => Ok(env.get_variable(var)?.1.clone()),
            Expr::MacroApp(macro_app) => macro_app.eval(env),
        };
        log::debug!("eval@#{} {}", env.current_context.index(), self);
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
