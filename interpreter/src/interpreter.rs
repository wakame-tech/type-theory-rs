use crate::{eval::Eval, interpreter_env::InterpreterEnv};
use anyhow::{Ok, Result};
use ast::{
    ast::{from_expr, Expr, FnApp, FnDef, Let, Program},
    into_ast::into_ast,
};
use rand::{distributions::Alphanumeric, Rng};
use symbolic_expressions::parser::parse_str;

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
        env.new_var(self.name.clone(), val.type_id, val);
        Ok(Expr::Variable(self.name.clone()))
    }
}

impl Eval for FnApp {
    fn eval(&self, env: &mut InterpreterEnv) -> Result<Expr> {
        let param = self
            .args
            .iter()
            .map(|arg| eval_expr(env, arg).and_then(|arg| from_expr(&arg)))
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
        let ret = eval_expr(&mut env, &fun.body);
        println!("{} env\n {}", self.fun, env);
        ret
    }
}

pub fn eval_expr(env: &mut InterpreterEnv, expr: &Expr) -> Result<Expr> {
    match expr {
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

pub fn interpret(env: &mut InterpreterEnv, sexps: &Vec<&str>) -> Result<()> {
    let sexps = sexps
        .iter()
        .map(|s| parse_str(s).map_err(|e| anyhow::anyhow!(e)))
        .collect::<Result<Vec<_>>>()?;

    let program = Program(
        sexps
            .iter()
            .map(|s| into_ast(&mut env.alloc, s))
            .collect::<Result<Vec<_>>>()?,
    );

    for expr in &program.0 {
        let evaluated = eval_expr(env, expr)?;
        println!("eval: {} -> {}", expr, &evaluated);
    }
    Ok(())
}
