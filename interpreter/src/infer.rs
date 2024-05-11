use crate::{interpreter_env::InterpreterEnv, traits::InferType};
use anyhow::Result;
use ast::ast::{Expr, FnApp, FnDef, Let, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use structural_typesystem::{
    type_alloc::TypeAlloc,
    type_env::TypeEnv,
    types::{Id, Type},
};
use symbolic_expressions::{parser::parse_str, Sexp};

impl InferType for Value {
    fn infer_type(&self, env: &mut InterpreterEnv, non_generic: &HashSet<Id>) -> Result<Id> {
        match self {
            Value::Nil => env.type_env.get(&parse_str("int")?),
            Value::Bool(_) => env.type_env.get(&parse_str("bool")?),
            Value::Number(_) => env.type_env.get(&parse_str("int")?),
            Value::Record(record) => {
                let record_type = record
                    .iter()
                    .map(|(k, v)| {
                        let ty_id = v.infer_type(env, non_generic)?;
                        Ok(Sexp::List(vec![
                            Sexp::String(k.to_string()),
                            env.type_env.alloc.as_sexp(ty_id)?,
                        ]))
                    })
                    .collect::<Result<Vec<_>>>()?;
                let record_type = Sexp::List(
                    vec![Sexp::String("record".to_string())]
                        .into_iter()
                        .chain(record_type)
                        .collect(),
                );
                env.type_env.new_type(&record_type)
            }
        }
    }
}

impl InferType for FnApp {
    fn infer_type(&self, env: &mut InterpreterEnv, non_generic: &HashSet<Id>) -> Result<Id> {
        let FnApp(f, v) = self;
        let fn_ty = f.infer_type(env, non_generic)?;
        let arg_ty_id = v.infer_type(env, non_generic)?;
        let ret_ty_id = env.type_env.alloc.issue_id();
        env.type_env.alloc.insert(Type::variable(ret_ty_id));
        let new_fn_ty = env.type_env.alloc.issue_id();
        env.type_env
            .alloc
            .insert(Type::function(new_fn_ty, arg_ty_id, ret_ty_id));
        log::debug!(
            "#{} = ? -> ? vs #{} = #{} -> #{}",
            fn_ty,
            new_fn_ty,
            arg_ty_id,
            ret_ty_id
        );
        unify(&mut env.type_env, new_fn_ty, fn_ty)?;
        Ok(prune(&mut env.type_env.alloc, ret_ty_id))
    }
}

impl InferType for FnDef {
    fn infer_type(&self, env: &mut InterpreterEnv, non_generic: &HashSet<Id>) -> Result<Id> {
        let FnDef { arg, body, .. } = self;
        let arg_ty = if let Some(typ) = &arg.typ {
            env.type_env.new_type(typ)?
        } else {
            let id = env.type_env.alloc.issue_id();
            env.type_env.alloc.insert(Type::variable(id));
            id
        };
        env.current_mut()
            .insert(&arg.name, arg_ty, Expr::Variable(arg.name.to_string()));
        let mut new_non_generic = non_generic.clone();
        new_non_generic.insert(arg_ty);
        let ret_ty = body.infer_type(env, &new_non_generic)?;
        let fn_ty = env.type_env.alloc.issue_id();
        env.type_env
            .alloc
            .insert(Type::function(fn_ty, arg_ty, ret_ty));

        // log::debug!("#{}: {}", fn_ty_id, new_env.type_env.type_name(fn_ty_id)?);
        Ok(fn_ty)
    }
}

impl InferType for Let {
    fn infer_type(&self, env: &mut InterpreterEnv, non_generic: &HashSet<Id>) -> Result<Id> {
        let Let { typ, value, .. } = self;
        if let Some(type_expr) = typ {
            env.type_env.alloc.from_sexp(type_expr)
        } else {
            let infer_ty = value.infer_type(env, non_generic)?;
            return Ok(infer_ty);
        }
    }
}

impl InferType for Expr {
    fn infer_type(&self, env: &mut InterpreterEnv, non_generic: &HashSet<Id>) -> Result<Id> {
        let ret = match self {
            Expr::Literal(value) => value.infer_type(env, non_generic),
            Expr::Variable(name) => {
                let (id, _) = env.current().get(name)?.clone();
                let ng = non_generic.iter().cloned().collect::<Vec<_>>();
                let ret = fresh(&mut env.type_env, id, &ng);
                Ok(ret)
            }
            Expr::FnApp(app) => app.infer_type(env, non_generic),
            Expr::FnDef(def) => def.infer_type(env, non_generic),
            Expr::Let(r#let) => r#let.infer_type(env, non_generic),
        }?;
        log::debug!("infer_type {} :: {}", self, env.type_env.type_name(ret)?);
        Ok(ret)
    }
}

fn fresh_rec(env: &mut TypeEnv, tp: Id, mappings: &mut HashMap<Id, Id>, non_generic: &[Id]) -> Id {
    let p = prune(&mut env.alloc, tp);
    match env.alloc.get(p).unwrap().clone() {
        Type::Variable { .. } => {
            if is_generic(&mut env.alloc, p, non_generic) {
                let insert = |env: &mut TypeEnv| {
                    let id = env.alloc.issue_id();
                    env.alloc.insert(Type::variable(id));
                    id
                };
                *mappings.entry(p).or_insert(insert(env))
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
    }
}

fn fresh(env: &mut TypeEnv, id: Id, non_generic: &[Id]) -> Id {
    log::debug!(
        "fresh #{} {} non_generic={:?}",
        id,
        env.alloc.as_sexp(id).unwrap(),
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
    let (a_ty, b_ty) = (env.alloc.get(a)?, env.alloc.get(b)?);
    log::debug!(
        "unify #{} = {} and #{} = {}",
        a,
        env.type_name(a)?,
        b,
        env.type_name(b)?
    );
    match (&a_ty, &b_ty) {
        (Type::Variable { .. }, _) => {
            if a != b {
                if occurs_in_type(&mut env.alloc, a, b) {
                    panic!("recursive unification")
                }
                log::debug!("type variable #{} := #{}", a, b);
                env.alloc.get_mut(a)?.set_instance(b);
                log::debug!("{:?}", env.alloc.get(a)?);
            }
            Ok(b)
        }
        (Type::Operator { .. }, Type::Variable { .. }) => unify(env, s, t),
        // unify fn type
        (
            Type::Operator {
                op: a_name,
                types: a_types,
                ..
            },
            Type::Operator {
                op: b_name,
                types: b_types,
                ..
            },
        ) if a_name == b_name => {
            let types = a_types
                .iter()
                .zip(b_types.iter())
                .map(|((label, a), (_, b))| Ok((label.clone(), unify(env, *a, *b)?)))
                .collect::<Result<BTreeMap<_, _>>>()?;
            let id = env.alloc.issue_id();
            let ty = Type::Operator {
                id,
                op: a_name.to_string(),
                types,
            };
            env.alloc.insert(ty);
            Ok(id)
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
    log::debug!("prune #{} {:?}", t, alloc.get(t).unwrap());
    match alloc.get(t) {
        Ok(Type::Variable {
            instance: Some(instance_id),
            ..
        }) => {
            let ty = alloc.get_mut(t).unwrap();
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
    if let Type::Operator { types, .. } = alloc.get(prune_t).unwrap().clone() {
        occurs_in(
            alloc,
            v,
            types.values().cloned().collect::<Vec<_>>().as_slice(),
        )
    } else {
        false
    }
}

#[cfg(test)]
mod test {
    use crate::interpreter_env::InterpreterEnv;
    use crate::tests::setup;
    use crate::traits::InferType;
    use anyhow::Result;
    use ast::into_ast::into_ast;
    use std::collections::HashSet;
    use symbolic_expressions::parser::parse_str;

    fn should_infer(env: &mut InterpreterEnv, expr: &str, type_expr: &str) -> Result<()> {
        setup();
        let exp = into_ast(&parse_str(expr)?)?;
        let infer_ty_id = exp.infer_type(env, &HashSet::new())?;
        assert_eq!(parse_str(type_expr)?, env.type_env.type_name(infer_ty_id)?);
        Ok(())
    }

    #[test]
    fn test_literal() -> Result<()> {
        let mut env = InterpreterEnv::default();
        should_infer(&mut env, "true", "bool")?;
        should_infer(&mut env, "1", "int")?;
        should_infer(&mut env, "(record (a 1))", "(record (a int))")?;
        Ok(())
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
        should_infer(&mut env, "(lam (x : bool) (not x))", "(-> bool bool)")
    }

    #[test]
    fn test_let_app() -> Result<()> {
        let mut env = InterpreterEnv::default();
        should_infer(&mut env, "(let a (id 1))", "int")
    }

    #[test]
    fn test_tvar() -> Result<()> {
        let mut env = InterpreterEnv::default();
        // should_infer(&mut env, "(id id)", "(-> a a)")
        should_infer(&mut env, "id", "(-> a a)")
    }

    #[test]
    fn test_lam_tvar() -> Result<()> {
        let mut env = InterpreterEnv::default();
        should_infer(&mut env, "(lam x (lam y x))", "(-> a (-> b a))")
    }
}
