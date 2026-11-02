use crate::{
    type_alloc::TypeAlloc,
    type_env::{container, record, TypeEnv},
    types::{Id, Type, LIST_TYPE_KEYWORD},
};
use anyhow::Result;
use ast::ast::{Case, Expr, FnApp, FnDef, Let, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

pub trait InferType {
    fn infer_type(&self, env: &mut TypeEnv, non_generic: &HashSet<Id>) -> Result<Id>;
}

impl InferType for Value {
    fn infer_type(&self, env: &mut TypeEnv, non_generic: &HashSet<Id>) -> Result<Id> {
        match self {
            Value::External(_) => Err(anyhow::anyhow!("external value")),
            Value::Bool(v) => env.new_type_str(if *v { "true" } else { "false" }),
            Value::Number(v) => env.new_type_str(v.to_string().as_str()),
            Value::Atom(v) => env.new_type_str(format!(":{}", v).as_str()),
            Value::String(v) => env.new_type_str(v.as_str()),
            Value::Record(fields) => {
                let fields = fields
                    .iter()
                    .map(|(k, v)| {
                        let ty_id = v.infer_type(env, non_generic)?;
                        Ok((k.to_string(), env.alloc.as_sexp(ty_id)?))
                    })
                    .collect::<Result<BTreeMap<_, _>>>()?;
                let record_type = record(fields);
                env.new_type(&record_type)
            }
            Value::List(elems) => {
                // element type is the join of all element types.
                // if empty, element type infers to a fresh type variable.
                let mut elem: Option<Id> = None;
                for e in elems {
                    let ty = e.infer_type(env, non_generic)?;
                    elem = Some(match elem {
                        Some(acc) => env.join(acc, ty)?,
                        None => ty,
                    });
                }
                let elem = match elem {
                    Some(elem) => elem,
                    None => env.new_type_str("a")?,
                };
                let elem_ty = env.alloc.as_sexp(elem)?;
                let container_ty = container(LIST_TYPE_KEYWORD.to_string(), vec![elem_ty]);
                env.new_type(&container_ty)
            }
        }
    }
}

impl InferType for FnApp {
    fn infer_type(&self, env: &mut TypeEnv, non_generic: &HashSet<Id>) -> Result<Id> {
        let FnApp(f, vs) = self;
        let fn_ty = f.infer_type(env, non_generic)?;
        let arg_ty_ids = vs
            .iter()
            .map(|v| v.infer_type(env, non_generic))
            .collect::<Result<Vec<_>>>()?;
        let ret_ty_id = env.alloc.issue_id();
        env.alloc.insert(Type::variable(ret_ty_id, None));
        let new_fn_ty = env.alloc.issue_id();
        env.alloc
            .insert(Type::function(new_fn_ty, arg_ty_ids.clone(), ret_ty_id));

        log::debug!(
            "\n([{}] -> {} #{})\n{}",
            arg_ty_ids
                .iter()
                .map(|id| env.alloc.debug(*id))
                .collect::<Result<Vec<_>>>()?
                .join(" "),
            env.alloc.debug(ret_ty_id)?,
            new_fn_ty,
            env.alloc.debug(fn_ty)?
        );

        unify(env, new_fn_ty, fn_ty)?;
        Ok(prune(&mut env.alloc, ret_ty_id))
    }
}

impl InferType for FnDef {
    fn infer_type(&self, env: &mut TypeEnv, non_generic: &HashSet<Id>) -> Result<Id> {
        let FnDef { args, body, .. } = self;
        let arg_tys = args
            .iter()
            .map(|arg| {
                let arg_ty = if let Some(typ) = &arg.typ {
                    env.new_type(typ)?
                } else {
                    let id = env.alloc.issue_id();
                    env.alloc.insert(Type::variable(id, None));
                    id
                };
                env.set_variable(&arg.name, arg_ty);
                Ok(arg_ty)
            })
            .collect::<Result<Vec<_>>>()?;
        let mut new_non_generic = non_generic.clone();
        new_non_generic.extend(arg_tys.iter());
        let ret_ty = body.infer_type(env, &new_non_generic)?;
        let fn_ty = env.alloc.issue_id();
        env.alloc.insert(Type::function(fn_ty, arg_tys, ret_ty));
        Ok(fn_ty)
    }
}

impl InferType for Let {
    fn infer_type(&self, env: &mut TypeEnv, non_generic: &HashSet<Id>) -> Result<Id> {
        let Let { typ, value, .. } = self;
        if let Some(type_expr) = typ {
            env.alloc.from_sexp(type_expr)
        } else {
            let infer_ty = value.infer_type(env, non_generic)?;
            Ok(infer_ty)
        }
    }
}

impl InferType for Case {
    fn infer_type(&self, env: &mut TypeEnv, non_generic: &HashSet<Id>) -> Result<Id> {
        let Case { branches, .. } = self;
        let mut ret_ty = None;
        for (_, body) in branches {
            let body_ty = body.infer_type(env, non_generic)?;
            if let Some(ret_ty) = ret_ty {
                unify(env, ret_ty, body_ty)?;
            } else {
                ret_ty = Some(body_ty);
            }
        }
        Ok(ret_ty.unwrap())
    }
}

impl InferType for Expr {
    fn infer_type(&self, env: &mut TypeEnv, non_generic: &HashSet<Id>) -> Result<Id> {
        let _span = tracing::debug_span!("infer", "{}", self).entered();
        let ret = match self {
            Expr::Literal(value) => value.infer_type(env, non_generic),
            Expr::Variable(name) => {
                let id = env.get_variable(name)?;
                let ng = non_generic.iter().cloned().collect::<Vec<_>>();
                let ret = fresh(env, id, &ng);
                Ok(ret)
            }
            Expr::FnApp(app) => app.infer_type(env, non_generic),
            Expr::FnDef(def) => def.infer_type(env, non_generic),
            Expr::Let(r#let) => r#let.infer_type(env, non_generic),
            Expr::TypeDef(type_def) => env.new_type(&type_def.typ),
            Expr::Case(case) => case.infer_type(env, non_generic),
            Expr::Include(_) => env.new_type_str("str"),
        }?;
        log::debug!(":{}", env.type_name(ret)?);
        Ok(ret)
    }
}

fn fresh_rec(env: &mut TypeEnv, tp: Id, mappings: &mut HashMap<Id, Id>, non_generic: &[Id]) -> Id {
    let p = prune(&mut env.alloc, tp);
    match env.alloc.get(p).unwrap() {
        Type::Variable { upper_bound, .. } => {
            if is_generic(&mut env.alloc, p, non_generic) {
                if let Some(id) = mappings.get(&p) {
                    *id
                } else {
                    let id = env.alloc.issue_id();
                    env.alloc.insert(Type::variable(id, upper_bound));
                    mappings.insert(p, id);
                    id
                }
            } else {
                p
            }
        }
        Type::Primitive { id, .. } => id,
        // rebuild composite types only when some child is instantiated,
        // so that ids of monomorphic types are preserved
        Type::Function { args, ret, .. } => {
            let new_args = args
                .iter()
                .map(|arg| fresh_rec(env, *arg, mappings, non_generic))
                .collect::<Vec<_>>();
            let new_ret = fresh_rec(env, ret, mappings, non_generic);
            if new_args == args && new_ret == ret {
                return p;
            }
            let id = env.alloc.issue_id();
            env.alloc.insert(Type::function(id, new_args, new_ret));
            id
        }
        Type::Record { fields, .. } => {
            let new_fields = fields
                .iter()
                .map(|(label, id)| (label.clone(), fresh_rec(env, *id, mappings, non_generic)))
                .collect::<BTreeMap<_, _>>();
            if new_fields == fields {
                return p;
            }
            let id = env.alloc.issue_id();
            env.alloc.insert(Type::record(id, new_fields));
            id
        }
        Type::Container { id, elements } => {
            let new_elements = elements
                .iter()
                .map(|id| fresh_rec(env, *id, mappings, non_generic))
                .collect::<Vec<_>>();
            if new_elements == elements {
                return p;
            }
            let new_id = env.alloc.issue_id();
            env.alloc.insert(Type::container(id, new_elements));
            new_id
        }
        Type::Union { types, .. } => {
            let new_types = types
                .iter()
                .map(|id| fresh_rec(env, *id, mappings, non_generic))
                .collect::<BTreeSet<_>>();
            if new_types == types {
                return p;
            }
            let id = env.alloc.issue_id();
            env.alloc.insert(Type::Union {
                id,
                types: new_types,
            });
            id
        }
    }
}

fn fresh(env: &mut TypeEnv, id: Id, non_generic: &[Id]) -> Id {
    // log::debug!(
    //     "fresh #{} {} non_generic={:?}",
    //     id,
    //     env.alloc.as_sexp(id).unwrap(),
    //     non_generic
    // );
    let mut mappings: HashMap<Id, Id> = HashMap::new();
    fresh_rec(env, id, &mut mappings, non_generic)
}

fn unify(env: &mut TypeEnv, t: Id, s: Id) -> Result<usize> {
    let (a, b) = (prune(&mut env.alloc, t), prune(&mut env.alloc, s));
    if a == b {
        return Ok(a);
    }
    // log::debug!("unify#left  {}", env.alloc.debug(a)?);
    // log::debug!("unify#right {}", env.alloc.debug(b)?);
    let (a_ty, b_ty) = (env.alloc.get(a)?, env.alloc.get(b)?);
    match (&a_ty, &b_ty) {
        (Type::Variable { .. }, _) => {
            if a != b {
                if occurs_in_type(&mut env.alloc, a, b) {
                    return Err(anyhow::anyhow!("recursive unification"));
                }
                // log::debug!("type variable #{} := #{}", a, b);
                env.alloc.get_mut(a)?.set_instance(b);
            }
            Ok(b)
        }
        (_, Type::Variable { .. }) => unify(env, s, t),
        // unify fn type
        (
            Type::Function {
                args: a_args,
                ret: a_ret,
                ..
            },
            Type::Function {
                args: b_args,
                ret: b_ret,
                ..
            },
        ) => {
            let args = a_args
                .iter()
                .zip(b_args.iter())
                .map(|(a_arg, b_arg)| unify(env, *a_arg, *b_arg))
                .collect::<Result<Vec<_>>>()?;
            let ret = unify(env, *a_ret, *b_ret)?;
            let id = env.alloc.issue_id();
            env.alloc.insert(Type::function(id, args, ret));
            Ok(id)
        }
        (
            Type::Record {
                fields: a_types, ..
            },
            Type::Record {
                fields: b_types, ..
            },
        ) => {
            let fields = a_types
                .iter()
                .zip(b_types.iter())
                .map(|((label, a), (_, b))| Ok((label.clone(), unify(env, *a, *b)?)))
                .collect::<Result<BTreeMap<_, _>>>()?;
            let id = env.alloc.issue_id();
            env.alloc.insert(Type::record(id, fields));
            Ok(id)
        }
        (
            Type::Container {
                elements: a_elements,
                id,
            },
            Type::Container {
                elements: b_elements,
                ..
            },
        ) => {
            let elements = a_elements
                .iter()
                .zip(b_elements.iter())
                .map(|(a, b)| unify(env, *a, *b))
                .collect::<Result<Vec<_>>>()?;
            env.alloc.insert(Type::container(*id, elements));
            Ok(*id)
        }
        _ => {
            if env.is_subtype(a, b)? {
                Ok(b)
            } else if env.is_subtype(b, a)? {
                Ok(a)
            } else {
                Err(anyhow::anyhow!(
                    "unify: type mismatch: {} #{} != {} #{}",
                    env.type_name(a)?,
                    a,
                    env.type_name(b)?,
                    b,
                ))
            }
        }
    }
}

/// returns an instance of t
fn prune(alloc: &mut TypeAlloc, t: Id) -> Id {
    // log::debug!("prune #{} {:?}", t, alloc.get(t).unwrap());
    match alloc.get(t) {
        Ok(Type::Variable {
            instance: Some(instance_id),
            ..
        }) => {
            let ty = alloc.get_mut(t).unwrap();
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
    match alloc.get(prune_t).unwrap() {
        Type::Primitive { id, .. } => occurs_in(alloc, id, &[]),
        Type::Function { args, ret, .. } => {
            let args_ret = args
                .iter()
                .cloned()
                .chain(std::iter::once(ret))
                .collect::<Vec<_>>();
            occurs_in(alloc, v, &args_ret)
        }
        Type::Record { fields, .. } => occurs_in(
            alloc,
            v,
            fields.values().cloned().collect::<Vec<_>>().as_slice(),
        ),
        _ => false,
    }
}

#[cfg(test)]
mod test {
    use crate::{infer::InferType, tests::setup, type_env::TypeEnv};
    use anyhow::Result;
    use ast::into_ast::into_ast;
    use std::collections::HashSet;
    use symbolic_expressions::parser::parse_str;

    fn should_infer(env: &mut TypeEnv, expr: &str, type_expr: &str) -> Result<()> {
        setup();

        let expected = parse_str(type_expr)?;
        let exp = into_ast(&parse_str(expr)?)?;
        let infer_ty_id = exp.infer_type(env, &HashSet::new())?;
        // log::debug!("{}", env.alloc.)
        let actual = env.type_name(infer_ty_id)?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_literal() -> Result<()> {
        let mut env = TypeEnv::default();
        should_infer(&mut env, "true", "true")?;
        should_infer(&mut env, "1", "1")?;
        should_infer(&mut env, "(record (a : 1))", "(record (a : 1))")?;
        Ok(())
    }

    #[test]
    fn test_vec() -> Result<()> {
        let mut env = TypeEnv::default();
        should_infer(&mut env, "(vec 1 1)", "(vec 1)")?;
        should_infer(&mut env, "(vec 1 2)", "(vec (| 1 2))")?;
        should_infer(&mut env, "(vec 1 :a 3)", "(vec (| 1 :a 3))")?;
        should_infer(
            &mut env,
            "(vec (vec 1) (vec 2))",
            "(vec (| (vec 1) (vec 2)))",
        )?;
        Ok(())
    }

    #[test]
    fn test_fn() -> Result<()> {
        let mut env = TypeEnv::default();
        should_infer(&mut env, "(fn (x : int) 1)", "((int) -> 1)")
    }

    #[test]
    fn test_app() -> Result<()> {
        let mut env = TypeEnv::default();
        let ty = env.new_type_str("((bool) -> bool)")?;
        env.set_variable("not", ty);
        should_infer(&mut env, "(not true)", "bool")
    }

    #[test]
    fn test_let_app() -> Result<()> {
        let mut env = TypeEnv::default();
        let ty = env.new_type_str("((a) -> a)")?;
        env.set_variable("id", ty);
        should_infer(&mut env, "(let a (id 1))", "1")
    }

    #[test]
    fn test_let_polymorphism() -> Result<()> {
        let mut env = TypeEnv::default();
        let ty = env.new_type_str("((a) -> a)")?;
        env.set_variable("id", ty);
        should_infer(&mut env, "(id 1)", "1")?;
        should_infer(&mut env, "(id true)", "true")?;
        should_infer(&mut env, "id", "((a) -> a)")
    }

    #[test]
    fn test_fresh_shares_type_variable() -> Result<()> {
        let mut env = TypeEnv::default();
        let ty = env.new_type_str("((a a) -> a)")?;
        env.set_variable("f", ty);
        should_infer(&mut env, "(f 1 1)", "1")
    }

    #[test]
    fn test_tvar() -> Result<()> {
        let mut env = TypeEnv::default();
        let ty = env.new_type_str("((a) -> a)")?;
        env.set_variable("id", ty);
        should_infer(&mut env, "id", "((a) -> a)")
    }

    #[test]
    fn test_fn_tvar() -> Result<()> {
        let mut env = TypeEnv::default();
        should_infer(&mut env, "(fn x y x)", "((a b) -> a))")
    }
}
