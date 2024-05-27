use crate::{environment::Environment, externals::eval_externals};
use anyhow::{anyhow, Ok, Result};
use ast::ast::{Case, Expr, FnApp, FnDef, Let, Parameter, Program, Value};
use std::collections::HashMap;
use structural_typesystem::{type_env::TypeEnv, types::Type};

pub trait Eval {
    fn eval(&self, t_env: &mut TypeEnv, env: Environment) -> Result<(Expr, Environment)>;
}

impl Eval for Value {
    fn eval(&self, t_env: &mut TypeEnv, env: Environment) -> Result<(Expr, Environment)> {
        match self {
            Value::Record(fields) => {
                let fields = fields
                    .iter()
                    .map(|(name, value)| {
                        value
                            .eval(t_env, env.clone())
                            .map(|t| (name.to_string(), t.0))
                    })
                    .collect::<Result<HashMap<_, _>>>()?;
                Ok((Expr::Literal(Value::Record(fields)), env))
            }
            Value::List(elements) => {
                let elements = elements
                    .iter()
                    .map(|value| value.eval(t_env, env.clone()).map(|t| t.0))
                    .collect::<Result<Vec<_>>>()?;
                Ok((Expr::Literal(Value::List(elements)), env))
            }
            v => Ok((Expr::Literal(v.clone()), env)),
        }
    }
}

impl Eval for FnDef {
    fn eval(&self, _t_env: &mut TypeEnv, env: Environment) -> Result<(Expr, Environment)> {
        Ok((Expr::FnDef(self.clone()), env))
    }
}

impl Eval for Let {
    /// (let a int 1)
    fn eval(&self, t_env: &mut TypeEnv, env: Environment) -> Result<(Expr, Environment)> {
        let (value, mut env) = self.value.eval(t_env, env)?;
        if let Expr::Literal(Value::External(name)) = value.clone() {
            let Some(typ) = self.typ.as_ref() else {
                return Err(anyhow!("type is required"));
            };
            let id = t_env.new_type(typ)?;
            let Type::Function { args, .. } = t_env.alloc.get(id)? else {
                return Err(anyhow!("type is not function"));
            };
            env.insert(
                &self.name,
                Expr::FnDef(FnDef::new(
                    args.iter()
                        .enumerate()
                        .map(|(i, arg)| {
                            Ok(Parameter::new(
                                format!("_{}", i),
                                Some(t_env.type_name(*arg)?),
                            ))
                        })
                        .collect::<Result<Vec<_>>>()?,
                    Box::new(Expr::Literal(Value::External(name))),
                )),
            );
        } else {
            env.insert(&self.name, *self.value.clone());
        }
        Ok((value, env))
    }
}

impl Eval for FnApp {
    fn eval(&self, t_env: &mut TypeEnv, env: Environment) -> Result<(Expr, Environment)> {
        let (f, mut env) = self.0.eval(t_env, env)?;
        let Expr::FnDef(FnDef { args: params, body }) = f else {
            return Err(anyhow!("{} is not function", self.0));
        };
        let mut args = vec![];
        for (param, arg) in params.iter().zip(self.1.iter()) {
            let (arg, _env) = arg.eval(t_env, env.clone())?;
            args.push(arg.clone());
            // env = new_env;
            env.insert(&param.name, arg);
        }

        if let Expr::Literal(Value::External(name)) = body.as_ref() {
            eval_externals(t_env, env, name, args)
        } else {
            body.eval(t_env, env)
        }
    }
}

impl Eval for Case {
    fn eval(&self, t_env: &mut TypeEnv, env: Environment) -> Result<(Expr, Environment)> {
        for (pattern, body) in &self.branches {
            let (pattern, env) = pattern.eval(t_env, env.clone())?;
            if pattern == Expr::Literal(Value::Bool(true)) {
                return body.eval(t_env, env);
            }
        }
        Err(anyhow!("unreachable in case"))
    }
}

impl Eval for Expr {
    fn eval(&self, t_env: &mut TypeEnv, env: Environment) -> Result<(Expr, Environment)> {
        let _span = tracing::debug_span!("", "{}", self).entered();
        let (res, env) = match self {
            Expr::FnDef(fndef) => fndef.eval(t_env, env),
            Expr::Let(r#let) => r#let.eval(t_env, env),
            Expr::FnApp(fnapp) => fnapp.eval(t_env, env),
            e @ Expr::Literal(Value::External(_)) => Ok((e.clone(), env)),
            Expr::Literal(lit) => lit.eval(t_env, env),
            Expr::Variable(var) => Ok((env.get(var)?.clone(), env)),
            Expr::Case(case) => case.eval(t_env, env),
            e @ Expr::TypeDef(_) => Ok((e.clone(), env)),
        }?;
        log::debug!("= {}", res);
        Ok((res, env))
    }
}

impl Eval for Program {
    fn eval(&self, t_env: &mut TypeEnv, env: Environment) -> Result<(Expr, Environment)> {
        let mut env = env;
        let mut last_expr = Expr::Literal(Value::Number(0));
        for expr in &self.0 {
            let (expr, new_env) = expr.eval(t_env, env)?;
            env = new_env;
            last_expr = expr;
        }
        Ok((last_expr, env))
    }
}

#[cfg(test)]
mod tests {
    use super::Eval;
    use crate::{environment::Environment, eval_prelude, parse, tests::setup};
    use anyhow::Result;
    use ast::into_ast::into_ast;
    use structural_typesystem::type_env::TypeEnv;
    use symbolic_expressions::parser::parse_str;

    fn should_eval(expr: &str, expected: &str) -> Result<()> {
        let expr = parse(expr)?;
        let expected = into_ast(&parse_str(expected)?)?;

        let mut type_env = TypeEnv::default();
        let env = Environment::new(None);
        let env = eval_prelude(&mut type_env, env)?;
        setup();
        let (evaluated, _) = expr.eval(&mut type_env, env)?;
        assert_eq!(evaluated, expected);
        Ok(())
    }

    #[test]
    fn test_nest_fn() -> Result<()> {
        should_eval(
            r#"(let g (fn x (fn y (+ x y))))
            ((g 1) ((g 2) 3))"#,
            "6",
        )
    }
}
