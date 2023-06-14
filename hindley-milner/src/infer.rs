use anyhow::Result;
use ast::ast::{Expr};
use std::collections::{HashMap, HashSet};
use structural_typesystem::{
    type_env::TypeEnv,
    types::{Id, Type},
};

/// Hindley-Milner type inference.
struct HMInferer;

impl HMInferer {
    pub fn analyse(&self, expr: &Expr, _env: &mut TypeEnv, _non_generic: &HashSet<Id>) -> Result<Id> {
        println!("expr = {:?}", expr);
        todo!();

        // match &expr {
        //     Expr::Literal(value) => Ok(value.type_id),
        //     Expr::Variable(name) => {
        //         if let Some(value) = env.get_id(name) {
        //             let ng = non_generic.iter().cloned().collect::<Vec<_>>();
        //             let ret = self.fresh(alloc, env, value, &ng);
        //             return Ok(ret);
        //         }
        //         Err(anyhow::anyhow!("type {} not found", name))
        //     }
        //     Expr::FnApp(FnApp(_, f, v)) => {
        //         let fn_type = self.analyse(f, alloc, env, non_generic)?;
        //         let arg_type = self.analyse(v, alloc, env, non_generic)?;
        //         let ret = alloc.new_variable();

        //         let new_fn_type = alloc.new_function(arg_type, ret);
        //         self.unify(&mut alloc.alloc, new_fn_type, fn_type)?;
        //         Ok(ret)
        //     }
        //     Expr::FnDef(FnDef { param, body, .. }) => {
        //         let arg = param.clone();
        //         let mut new_env = env.clone();
        //         new_env.add(&arg.name, arg.typ_id);

        //         let mut new_non_generic = non_generic.clone();
        //         new_non_generic.insert(arg.typ_id);
        //         let ret = self.analyse(body, alloc, &mut new_env, &new_non_generic)?;
        //         Ok(alloc.new_function(arg.typ_id, ret))
        //     }
        //     Expr::Let(Let {
        //         name: _,
        //         type_id,
        //         value: _,
        //     }) => {
        //         Ok(*type_id)
        //         // if let Some(id) = type_id {
        //         //     return Ok(*id);
        //         // } else {
        //         //     let infer_id = self.analyse(value, alloc, env, non_generic)?;

        //         //     // let mut new_env = env.clone();
        //         //     // new_env.id_map.insert(name.clone(), infer_id);
        //         //     // return Ok(infer_id);

        //         //     env.register(name.as_str(), infer_id);
        //         //     return Ok(infer_id);
        //         // }
        //     }
        // }
    }

    fn fresh_rec(
        &self,
        env: &mut TypeEnv,
        tp: Id,
        mappings: &mut HashMap<Id, Id>,
        non_generic: &[Id],
    ) -> Id {
        let p = self.prune(&mut env.alloc.alloc, tp);
        match env.alloc.alloc.get(p).unwrap().clone() {
            Type::Variable { .. } => {
                if self.is_generic(&mut env.alloc.alloc, p, non_generic) {
                    *mappings.entry(p).or_insert(env.alloc.new_variable())
                } else {
                    p
                }
            }
            Type::Operator {
                ref name, types, ..
            } => {
                let ids = types
                    .iter()
                    .map(|t| self.fresh_rec(env, *t, mappings, non_generic))
                    .collect::<Vec<_>>();

                env.alloc.new_operator(name, &ids)
            }
            Type::Record { .. } => {
                todo!()
            }
        }
    }

    fn fresh(&self, env: &mut TypeEnv, t: Id, non_generic: &[Id]) -> Id {
        println!("fresh {} {:?}", t, non_generic);
        let mut mappings: HashMap<Id, Id> = HashMap::new();
        self.fresh_rec(env, t, &mut mappings, non_generic)
    }

    /// 単一化: 2つの型が一致するようななるべく小さな型代入を見つける
    fn unify(&self, alloc: &mut Vec<Type>, t: Id, s: Id) -> Result<()> {
        let (a, b) = (self.prune(alloc, t), self.prune(alloc, s));
        match (alloc.get(a).unwrap().clone(), alloc.get(b).unwrap().clone()) {
            (Type::Variable { .. }, _) => {
                if a != b {
                    if self.occurs_in_type(alloc, a, b) {
                        panic!("recursive unification")
                    }
                    alloc.get_mut(a).unwrap().set_instance(b);
                }
                Ok(())
            }
            (Type::Operator { .. }, Type::Variable { .. }) => self.unify(alloc, s, t),
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
                    .try_for_each(|(aa, bb)| self.unify(alloc, *aa, *bb))
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
            _ => Err(anyhow::anyhow!("type mismatch: {} != {}", a, b)),
        }
    }

    /// returns an instance of t
    fn prune(&self, alloc: &mut Vec<Type>, t: Id) -> Id {
        if let Type::Variable { instance, .. } = alloc.clone().get_mut(t).unwrap() {
            println!("prune {:?}", instance);
            if instance.is_some() {
                *instance = Some(self.prune(alloc, instance.unwrap()));
                return instance.unwrap();
            }
        }
        t
    }

    fn is_generic(&self, alloc: &mut Vec<Type>, id: Id, non_generic: &[Id]) -> bool {
        !self.occurs_in(alloc, id, non_generic)
    }

    fn occurs_in(&self, alloc: &mut Vec<Type>, id: Id, types: &[Id]) -> bool {
        types.iter().any(|t| self.occurs_in_type(alloc, id, *t))
    }

    /// typ 中に type_var が含まれているか
    fn occurs_in_type(&self, alloc: &mut Vec<Type>, v: Id, t: Id) -> bool {
        let prune_t = self.prune(alloc, t);
        if prune_t == v {
            return true;
        }
        if let Type::Operator { types, .. } = alloc.get(prune_t).unwrap().clone() {
            self.occurs_in(alloc, v, &types)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test {
    use crate::infer::HMInferer;
    use anyhow::Result;
    use ast::into_ast::into_ast;
    use log::LevelFilter;
    use std::io::Write;
    use std::{collections::HashSet, sync::Once};
    use structural_typesystem::type_env::TypeEnv;
    use symbolic_expressions::parser::parse_str;

    static INIT: Once = Once::new();

    fn setup() {
        INIT.call_once(|| {
            let _ = env_logger::builder()
                .is_test(true)
                .filter_level(LevelFilter::Debug)
                .format(|buf, record| writeln!(buf, "{}", record.args()))
                .try_init();
        });
    }

    fn should_infer(expr: &str, type_expr: &str) -> Result<()> {
        setup();
        let mut type_env = TypeEnv::default();
        let inferer = HMInferer;
        let exp = into_ast(&parse_str(expr)?)?;
        let infer_type_id = inferer.analyse(&exp, &mut type_env, &HashSet::new())?;
        let type_id = type_env.get(type_expr)?;
        assert_eq!(infer_type_id, type_id);
        Ok(())
    }

    #[test]
    fn test_var() -> Result<()> {
        should_infer("true", "bool")
    }

    #[test]
    fn test_lambda() -> Result<()> {
        should_infer("(lam (x : int) 1)", "(a -> int)")
    }

    #[test]
    fn test_app() -> Result<()> {
        should_infer("(not true)", "bool")
    }

    #[test]
    fn test_not() -> Result<()> {
        should_infer("(lam (x : bool) (not x))", "(bool -> bool)")
    }

    #[test]
    fn test_let_app() -> Result<()> {
        should_infer("(let a (succ 1))", "int")
    }

    #[test]
    fn test_iszero() -> Result<()> {
        should_infer("(zero? 0)", "bool")
    }

    #[test]
    fn test_tvar() -> Result<()> {
        should_infer("(id id)", "(a -> a)")
    }

    #[test]
    fn test_lam_tvar() -> Result<()> {
        should_infer("(lam (x : *) (lam (y : *) x))", "(a -> (b -> a))")
    }
}
