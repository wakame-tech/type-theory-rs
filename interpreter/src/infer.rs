use anyhow::Result;
use ast::ast::{Expr, FnApp, FnDef, Let, Value};
use std::collections::{HashMap, HashSet};
use structural_typesystem::{
    type_alloc::TypeAlloc,
    type_env::TypeEnv,
    types::{Id, Type},
};
use symbolic_expressions::{parser::parse_str, Sexp};

use crate::interpreter_env::InterpreterEnv;

pub fn infer_value_type(type_env: &TypeEnv, value: &Value) -> Result<Id> {
    match &value.raw {
        Sexp::String(lit) if lit.parse::<usize>().is_ok() => type_env.get(&parse_str("int")?),
        Sexp::String(lit) if lit == "true" || lit == "false" => type_env.get(&parse_str("bool")?),
        sexp => Err(anyhow::anyhow!("unknown literal {}", sexp)),
    }
}

pub fn infer_type(env: &mut InterpreterEnv, expr: &Expr, non_generic: &HashSet<Id>) -> Result<Id> {
    let ret = match &expr {
        Expr::Literal(value) => infer_value_type(&env.type_env, value),
        Expr::Variable(name) => {
            let (id, _) = env.get_variable(name)?.clone();
            let ng = non_generic.iter().cloned().collect::<Vec<_>>();
            let ret = fresh(&mut env.type_env, id, &ng);
            Ok(ret)
        }
        Expr::FnApp(FnApp(f, v)) => {
            let fn_ty = infer_type(env, f, non_generic)?;
            let arg_ty_id = infer_type(env, v, non_generic)?;
            let ret_ty_id = env.type_env.alloc.new_variable();
            let new_fn_ty = env.type_env.alloc.new_function(arg_ty_id, ret_ty_id);
            log::debug!(
                "#{} = . -> . vs #{} = #{} -> #{}",
                fn_ty,
                new_fn_ty,
                arg_ty_id,
                ret_ty_id
            );
            unify(&mut env.type_env, new_fn_ty, fn_ty)?;
            Ok(prune(&mut env.type_env.alloc, ret_ty_id))
        }
        Expr::FnDef(FnDef { param, body, .. }) => {
            // TODO
            let mut new_env = env.clone();
            let arg_ty = new_env.type_env.new_type(&param.typ)?;

            let mut new_non_generic = non_generic.clone();
            new_non_generic.insert(arg_ty);
            let ret_ty = infer_type(&mut new_env, body, &new_non_generic)?;
            let fn_ty = Sexp::List(vec![
                Sexp::String("->".to_string()),
                env.type_env
                    .alloc
                    .as_sexp(arg_ty, &mut Default::default())?,
                env.type_env
                    .alloc
                    .as_sexp(ret_ty, &mut Default::default())?,
            ]);

            let fn_ty_id = env.type_env.new_type(&fn_ty)?;
            Ok(fn_ty_id)
        }
        Expr::Let(Let { typ, value, .. }) => {
            if let Some(type_expr) = typ {
                env.type_env.alloc.from_sexp(type_expr)
            } else {
                let infer_ty = infer_type(env, value, non_generic)?;
                return Ok(infer_ty);
            }
        }
        Expr::MacroApp(..) => todo!(),
    }?;
    log::debug!(
        "infer_type {} :: #{} {}",
        expr,
        ret,
        env.type_env.alloc.as_sexp(ret, &mut Default::default())?
    );
    Ok(ret)
}

fn fresh_rec(env: &mut TypeEnv, tp: Id, mappings: &mut HashMap<Id, Id>, non_generic: &[Id]) -> Id {
    let p = prune(&mut env.alloc, tp);
    match env.alloc.from_id(p).unwrap().clone() {
        Type::Variable { .. } => {
            if is_generic(&mut env.alloc, p, non_generic) {
                *mappings.entry(p).or_insert(env.alloc.new_variable())
            } else {
                p
            }
        }
        Type::Operator { id, .. } => {
            // let ids = types
            //     .iter()
            //     .map(|t| self.fresh_rec(env, *t, mappings, non_generic))
            // .collect::<Vec<_>>();
            // env.alloc.new_operator(name, &ids)
            id
        }
        Type::Record { .. } => {
            todo!()
        }
    }
}

fn fresh(env: &mut TypeEnv, id: Id, non_generic: &[Id]) -> Id {
    log::debug!(
        "fresh #{} {} non_generic={:?}",
        id,
        env.alloc.as_sexp(id, &mut Default::default()).unwrap(),
        non_generic
    );
    let mut mappings: HashMap<Id, Id> = HashMap::new();
    fresh_rec(env, id, &mut mappings, non_generic)
}

fn unify(env: &mut TypeEnv, t: Id, s: Id) -> Result<usize> {
    let (a, b) = (prune(&mut env.alloc, t), prune(&mut env.alloc, s));
    if a == b {
        return Ok(a);
    }
    let (a_ty, b_ty) = (env.alloc.from_id(a)?, env.alloc.from_id(b)?);
    log::debug!(
        "unify #{} {} and #{} {}",
        a,
        env.alloc.as_sexp(a, &mut Default::default())?,
        b,
        env.alloc.as_sexp(b, &mut Default::default())?,
    );
    match (&a_ty, &b_ty) {
        (Type::Variable { .. }, _) => {
            if a != b {
                if occurs_in_type(&mut env.alloc, a, b) {
                    panic!("recursive unification")
                }
                log::debug!("type variable #{} := #{}", a, b);
                env.alloc.from_id_mut(a)?.set_instance(b);
                log::debug!("{:?}", env.alloc.from_id(a)?);
            }
            Ok(b)
        }
        (Type::Operator { .. }, Type::Variable { .. }) => unify(env, s, t),
        // unify fn type
        (
            Type::Operator {
                name: a_name,
                types: a_types,
                ..
            },
            Type::Operator {
                name: b_name,
                types: b_types,
                ..
            },
        ) if a_name == "->" && b_name == "->" && a_types.len() == 2 && b_types.len() == 2 => {
            let param_ty_id = unify(env, a_types[0], b_types[0])?;
            let ret_ty_id = unify(env, a_types[1], b_types[1])?;
            let fn_ty = Sexp::List(vec![
                Sexp::String("->".to_string()),
                env.alloc.as_sexp(param_ty_id, &mut Default::default())?,
                env.alloc.as_sexp(ret_ty_id, &mut Default::default())?,
            ]);
            env.new_type(&fn_ty)
        }
        (
            Type::Record {
                id: _a_id,
                types: _a_types,
            },
            Type::Record {
                id: _b_id,
                types: _b_types,
            },
        ) => {
            todo!()
        }
        _ => Err(anyhow::anyhow!(
            "unify: type mismatch: {:?} != {:?}",
            a_ty,
            b_ty
        )),
    }
}

/// returns an instance of t
fn prune(alloc: &mut TypeAlloc, t: Id) -> Id {
    log::debug!("prune #{} {:?}", t, alloc.from_id(t).unwrap());
    match alloc.from_id(t) {
        Ok(Type::Variable {
            instance: Some(instance_id),
            ..
        }) => {
            let ty = alloc.from_id_mut(t).unwrap();
            log::debug!("prune {:?}", ty);
            ty.set_instance(instance_id);
            instance_id
        }
        _ => t,
    }
}

fn is_generic(alloc: &mut TypeAlloc, id: Id, non_generic: &[Id]) -> bool {
    !occurs_in(alloc, id, non_generic)
}

fn occurs_in(alloc: &mut TypeAlloc, id: Id, types: &[Id]) -> bool {
    types.iter().any(|t| occurs_in_type(alloc, id, *t))
}

/// includes type variables in `t`
fn occurs_in_type(alloc: &mut TypeAlloc, v: Id, t: Id) -> bool {
    let prune_t = prune(alloc, t);
    if prune_t == v {
        return true;
    }
    if let Type::Operator { types, .. } = alloc.from_id(prune_t).unwrap().clone() {
        occurs_in(alloc, v, &types)
    } else {
        false
    }
}

#[cfg(test)]
mod test {
    use crate::infer::infer_type;
    use crate::interpreter_env::InterpreterEnv;
    use crate::tests::setup;
    use anyhow::Result;
    use ast::ast::Expr;
    use ast::into_ast::into_ast;
    use std::collections::HashSet;
    use symbolic_expressions::parser::parse_str;

    fn should_infer(env: &mut InterpreterEnv, expr: &str, type_expr: &str) -> Result<()> {
        setup();
        let exp = into_ast(&parse_str(expr)?)?;
        let infer_ty_id = infer_type(env, &exp, &HashSet::new())?;
        log::debug!(
            "infer_ty {} (id={})",
            env.type_env
                .alloc
                .as_sexp(infer_ty_id, &mut Default::default())?,
            infer_ty_id
        );
        let expect_ty_id = env.type_env.get(&parse_str(type_expr)?)?;

        assert_eq!(
            expect_ty_id,
            infer_ty_id,
            "\nexpect {} but {}",
            env.type_env
                .alloc
                .as_sexp(expect_ty_id, &mut Default::default())?,
            env.type_env
                .alloc
                .as_sexp(infer_ty_id, &mut Default::default())?,
        );
        Ok(())
    }

    #[test]
    fn test_var() -> Result<()> {
        let mut env = InterpreterEnv::default();
        should_infer(&mut env, "true", "bool")
    }

    #[test]
    fn test_lambda() -> Result<()> {
        let mut env = InterpreterEnv::default();
        should_infer(&mut env, "(lam (x : int) 1)", "(-> int int)")
    }

    #[test]
    fn test_app() -> Result<()> {
        let mut env = InterpreterEnv::default();
        should_infer(&mut env, "(not true)", "bool")
    }

    #[test]
    fn test_not() -> Result<()> {
        let mut env = InterpreterEnv::default();
        env.new_var(
            "x",
            Expr::Variable("x".to_string()),
            env.type_env.get(&parse_str("bool")?)?,
        );
        should_infer(&mut env, "(lam (x : bool) (not x))", "(-> bool bool)")
    }

    #[test]
    fn test_let_app() -> Result<()> {
        let mut env = InterpreterEnv::default();
        should_infer(&mut env, "(let a (succ 1))", "int")
    }

    #[test]
    fn test_tvar() -> Result<()> {
        let mut env = InterpreterEnv::default();
        should_infer(&mut env, "(id id)", "(-> a a)")
    }

    #[test]
    fn test_lam_tvar() -> Result<()> {
        let mut env = InterpreterEnv::default();
        should_infer(&mut env, "(lam x (lam y x))", "(-> a (-> b a))")
    }
}
