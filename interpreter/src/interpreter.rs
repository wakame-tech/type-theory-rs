use anyhow::Result;
use symbolic_expressions::{parser::parse_str, Sexp};

use crate::{
    ast::{Expr, Program},
    env::Env,
    into_ast::into_ast,
};

pub fn eval_expr(env: &mut Env, expr: &Expr) -> Result<Expr> {
    match expr {
        Expr::FnDef(fndef) => {
            todo!();
        }
        Expr::FnApp(fnapp) => match fnapp.name.as_str() {
            "+" => {
                let (lhs, rhs) = (
                    eval_expr(env, &fnapp.args[0])?,
                    eval_expr(env, &fnapp.args[1])?,
                );
                if !matches!(lhs, Expr::Literal(_)) || !matches!(rhs, Expr::Literal(_)) {
                    return Err(anyhow::anyhow!("lhs or rhs is not literal"));
                }
                Ok(Expr::Literal(lhs.literal()? + rhs.literal()?))
            }
            _ => todo!(),
        },
        Expr::Literal(lit) => Ok(Expr::Literal(*lit)),
        Expr::Variable(var) => {
            if let Some(v) = env.variables.get(var) {
                Ok(Expr::Literal(*v))
            } else {
                Err(anyhow::anyhow!("variable {} not found", var))
            }
        }
    }
}

pub fn interpret(env: &mut Env, sexps: &Vec<&str>) -> Result<()> {
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
        println!("{:?} -> {:?}", expr, &evaluated);
    }
    Ok(())
}
