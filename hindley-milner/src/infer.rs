use anyhow::Result;
use std::collections::{HashMap, HashSet};
use structural_typesystem::{
    issuer::{new_function, new_operator, new_variable},
    type_env::TypeEnv,
    types::{Id, Type},
};
use symbolic_expressions::Sexp;

fn is_number(lit: &str) -> bool {
    lit.chars().all(|c| c.is_numeric())
}

fn get_type(a: &mut Vec<Type>, name: &str, env: &TypeEnv, non_generic: &HashSet<Id>) -> Id {
    if let Some(value) = env.0.get(name) {
        let ng = non_generic.iter().cloned().collect::<Vec<_>>();
        fresh(a, *value, &ng)
    } else if is_number(name) {
        // int
        0
    } else {
        panic!("unknown symbol: {}", name)
    }
}

pub fn analyse(
    alloc: &mut Vec<Type>,
    expr: &Sexp,
    env: &TypeEnv,
    non_generic: &HashSet<Id>,
) -> Result<Id> {
    let ret = match &expr {
        Sexp::String(ref name) => Ok(get_type(alloc, name, env, non_generic)),
        Sexp::List(opes) => {
            let op = opes[0].string()?;
            match op.as_str() {
                "app" => {
                    let (func, arg) = (&opes[1], &opes[2]);
                    let fn_type = analyse(alloc, func, env, non_generic)?;
                    let arg_type = analyse(alloc, arg, env, non_generic)?;
                    let ret = new_variable(alloc);
                    let new_fn_type = new_function(alloc, arg_type, ret.clone());
                    unify(alloc, new_fn_type, fn_type)?;
                    Ok(ret)
                }
                "lam" => {
                    let (arg, body) = (opes[1].string()?, &opes[2]);
                    let arg_type_id = new_variable(alloc);
                    let mut new_env = env.clone();
                    new_env.0.insert(arg.clone(), arg_type_id);

                    let mut new_non_generic = non_generic.clone();
                    new_non_generic.insert(arg_type_id);
                    let ret = analyse(alloc, body, &new_env, &new_non_generic)?;
                    Ok(new_function(alloc, arg_type_id, ret))
                }
                "let" => {
                    let (v, defn, body) = (&opes[1], &opes[2], &opes[3]);
                    let defn_type_id = analyse(alloc, defn, env, non_generic)?;
                    let mut new_env = env.clone();
                    new_env
                        .0
                        .insert(v.string().unwrap().to_string(), defn_type_id);
                    analyse(alloc, body, &new_env, non_generic)
                }
                _ => Err(anyhow::anyhow!("unknown operator: {}", op)),
            }
        }
        Sexp::Empty => Err(anyhow::anyhow!("empty expression")),
    };
    ret
}

fn fresh(alloc: &mut Vec<Type>, t: Id, non_generic: &[Id]) -> Id {
    let mut mappings: HashMap<Id, Id> = HashMap::new();

    fn fresh_rec(
        alloc: &mut Vec<Type>,
        tp: Id,
        mappings: &mut HashMap<Id, Id>,
        non_generic: &[Id],
    ) -> Id {
        let p = prune(alloc, tp);
        match alloc.get(p).unwrap().clone() {
            Type::Variable { .. } => {
                if is_generic(alloc, p, non_generic) {
                    mappings.entry(p).or_insert(new_variable(alloc)).clone()
                } else {
                    p
                }
            }
            Type::Operator {
                ref name, types, ..
            } => {
                let ids = types
                    .iter()
                    .map(|t| fresh_rec(alloc, *t, mappings, non_generic))
                    .collect::<Vec<_>>();
                new_operator(alloc, name, &ids)
            }
        }
    }

    fresh_rec(alloc, t, &mut mappings, non_generic)
}

/// 単一化: 2つの型が一致するようななるべく小さな型代入を見つける
fn unify(alloc: &mut Vec<Type>, t: Id, s: Id) -> Result<()> {
    let (a, b) = (prune(alloc, t), prune(alloc, s));
    match (alloc.get(a).unwrap().clone(), alloc.get(b).unwrap().clone()) {
        (Type::Variable { .. }, _) => {
            if a != b {
                if occurs_in_type(alloc, a, b) {
                    panic!("recursive unification")
                }
                alloc.get_mut(a).unwrap().set_instance(b);
            }
            Ok(())
        }
        (Type::Operator { .. }, Type::Variable { .. }) => unify(alloc, s, t),
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
        ) => {
            if a_name != b_name || a_types.len() != b_types.len() {
                return Err(anyhow::anyhow!("type mismatch: {} != {}", a_name, b_name));
            }
            a_types
                .iter()
                .zip(b_types.iter())
                .map(|(aa, bb)| unify(alloc, *aa, *bb))
                .collect::<Result<_>>()
        }
    }
}

/// returns an instance of t
fn prune(alloc: &mut Vec<Type>, t: Id) -> Id {
    if let Type::Variable { instance, .. } = alloc.clone().get_mut(t).unwrap() {
        if instance.is_some() {
            *instance = Some(prune(alloc, instance.unwrap()));
            return instance.unwrap();
        }
    }
    t
}

fn is_generic(alloc: &mut Vec<Type>, id: Id, non_generic: &[Id]) -> bool {
    !occurs_in(alloc, id, non_generic)
}

fn occurs_in(alloc: &mut Vec<Type>, id: Id, types: &[Id]) -> bool {
    types.iter().any(|t| occurs_in_type(alloc, id, *t))
}

/// typ 中に type_var が含まれているか
fn occurs_in_type(alloc: &mut Vec<Type>, v: Id, t: Id) -> bool {
    let prune_t = prune(alloc, t);
    if prune_t == v {
        return true;
    }
    if let Type::Operator { types, .. } = alloc.get(prune_t).unwrap().clone() {
        return occurs_in(alloc, v, &types);
    } else {
        return false;
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use anyhow::Result;
    use structural_typesystem::{issuer::Issuer, type_env::default_env};
    use symbolic_expressions::parser::parse_str;

    use crate::infer::analyse;

    fn should_infer(expr: &str, typ: &str) -> Result<()> {
        let (mut alloc, env) = default_env();
        let exp = parse_str(expr)?;
        let id = analyse(&mut alloc, &exp, &env, &HashSet::new())?;
        assert_eq!(alloc[id].as_string(&alloc, &mut Issuer::new('a')), typ);
        Ok(())
    }

    #[test]
    fn test_var() -> Result<()> {
        should_infer("true", "bool")
    }

    #[test]
    fn test_lambda() -> Result<()> {
        should_infer("(lam x 1)", "(a -> int)")
    }

    #[test]
    fn test_app() -> Result<()> {
        should_infer("(app not true)", "bool")
    }

    #[test]
    fn test_not() -> Result<()> {
        should_infer("(lam x (app not x))", "(bool -> bool)")
    }

    #[test]
    fn test_let_app() -> Result<()> {
        should_infer("(let a (app succ 1) a)", "int")
    }

    #[test]
    fn test_tvar() -> Result<()> {
        should_infer("(app id id)", "(a -> a)")
    }

    #[test]
    fn test_lam_tvar() -> Result<()> {
        should_infer("(lam x (lam y x))", "(a -> (b -> a))")
    }
}
