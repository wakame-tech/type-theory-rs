use anyhow::{Ok, Result};
use rand::{distributions::Alphanumeric, Rng};
use symbolic_expressions::parser::parse_str;

use crate::{
    ast::{from_expr, Eval, Expr, FnApp, FnDef, IntrinsicFn, Let, Program, Value},
    interpreter_env::InterpreterEnv,
    into_ast::into_ast,
};

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
        let val = eval_expr(env, &self.value)?.literal()?;
        env.new_var(self.name.clone(), self.typ_id, val);
        Ok(Expr::Variable(self.name.clone()))
    }
}

impl Eval for FnApp {
    // (+ 1 2) => (app + 1 2) => 3
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr> {
        let param = self
            .args
            .iter()
            .map(|arg| eval_expr(env, arg).and_then(|arg| from_expr(&arg)))
            .collect::<Result<Vec<_>>>()?;
        if let Some(fun) = match &*self.fun {
            Expr::Variable(fn_name) => env.functions.get(fn_name),
            Expr::FnDef(fndef) => {
                let name = fndef.eval(env)?.name()?;
                env.functions.get(&name)
            }
            _ => return Err(anyhow::anyhow!("{} is not callable", self.fun)),
        } {
            let mut env = env.clone();
            // intrinsic function
            match fun.intrinsic {
                Some(IntrinsicFn::Add) => Ok(Expr::Literal(Value::Int(
                    param[0].as_int()? + param[1].as_int()?,
                ))),
                Some(IntrinsicFn::Eq) => Ok(Expr::Literal(Value::Bool(
                    param[0].as_int()? == param[1].as_int()?,
                ))),
                Some(IntrinsicFn::IsZero) => {
                    Ok(Expr::Literal(Value::Bool(param[0].as_int()? == 0)))
                }
                None => {
                    for (param, val) in fun.params.iter().zip(param) {
                        env.new_var(param.name.clone(), param.typ_id.clone(), val);
                    }
                    let ret = eval_expr(&mut env, &fun.body);
                    println!("{} env\n {}", self.fun, env);
                    ret
                }
            }
        } else {
            return Err(anyhow::anyhow!("{} is not found", self.fun));
        }
    }
}

pub fn eval_expr(env: &mut InterpreterEnv, expr: &Expr) -> Result<Expr> {
    match expr {
        Expr::FnDef(fndef) => fndef.eval(env),
        Expr::Let(r#let) => r#let.eval(env),
        Expr::FnApp(fnapp) => fnapp.eval(env),
        Expr::Literal(lit) => Ok(Expr::Literal(*lit)),
        Expr::Variable(var) => {
            if let Some((_, v)) = env.variables.get(var) {
                Ok(Expr::Literal(*v))
            } else {
                Err(anyhow::anyhow!("variable {} not found", var))
            }
        }
    }
}

pub fn interpret(env: &mut InterpreterEnv, sexps: &Vec<&str>) -> Result<()> {
    let sexps = sexps
        .iter()
        .map(|s| parse_str(s).map_err(|e| anyhow::anyhow!(e)))
        .collect::<Result<Vec<_>>>()?;

    let program = Program(
        sexps
            .iter()
            .map(|s| into_ast(env, s))
            .collect::<Result<Vec<_>>>()?,
    );

    for expr in &program.0 {
        let evaluated = eval_expr(env, expr)?;
        println!("eval: {} -> {}", expr, &evaluated);
    }
    Ok(())
}
