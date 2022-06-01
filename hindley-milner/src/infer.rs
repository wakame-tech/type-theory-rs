use std::collections::HashMap;

use crate::types::{type_var, Type};
use anyhow::{anyhow, Result};
use symbolic_expressions::Sexp;

pub fn default_env() -> HashMap<String, Type> {
    let a = Box::new(type_var().1);

    HashMap::from([
        ("true".to_string(), Type::bool()),
        ("false".to_string(), Type::bool()),
        ("not".to_string(), Type::bool().to(Type::bool())),
        ("id".to_string(), a.clone().to(*a)),
        ("zero?".to_string(), Type::int().to(Type::bool())),
    ])
}

#[derive(Debug)]
pub struct InferCtx {
    pub env: HashMap<String, Type>,
    pub references: HashMap<String, Type>,
}

fn is_number(lit: &String) -> bool {
    lit.chars().all(|c| c.is_numeric())
}

impl InferCtx {
    pub fn new() -> Self {
        Self {
            env: default_env(),
            references: HashMap::new(),
        }
    }

    pub fn copy(&self) -> Self {
        Self {
            env: self.env.clone(),
            references: self.references.clone(),
        }
    }

    pub fn update(&mut self, name: &String) -> Option<Type> {
        if let Some(typ) = self.references.get(name) {
            let typ = self.fresh(typ);
            self.env.insert(name.clone(), typ.clone());
            return Some(typ.clone());
        }
        None
    }

    fn fresh_rec(&self, mapping: &mut HashMap<Type, Type>, t: &Type) -> Type {
        let u = prune(t);
        match &u {
            Type::TypeVar(_, Some(name)) => {
                if is_generic(self, &u) {
                    if !mapping.contains_key(&u) {
                        let (_, t) = type_var();
                        mapping.insert(u.clone(), t);
                    }
                    return mapping.get(&u).unwrap().clone();
                } else {
                    return u;
                }
            }
            Type::Lambda(arg_typ, ret_type) => {
                let arg_typ = self.fresh_rec(mapping, arg_typ);
                let ret = self.fresh_rec(mapping, ret_type);
                return Type::Lambda(Box::new(arg_typ), Box::new(ret));
            }
            _ => u,
        }
    }

    fn fresh(&self, typ: &Type) -> Type {
        let mut mapping: HashMap<Type, Type> = HashMap::new();
        return self.fresh_rec(&mut mapping, typ);
    }
}

pub fn infer(ctx: &mut InferCtx, expr: Sexp) -> Result<Type> {
    let ret = match &expr {
        Sexp::String(var) => {
            if let Some(typ) = ctx.env.get(var) {
                return Ok(typ.clone());
            }
            if let Some(typ) = ctx.update(var) {
                Ok(typ)
            } else if is_number(&var) {
                Ok(Type::int())
            } else {
                Err(anyhow!("unknown variable: {}", var))
            }
        }
        Sexp::List(opes) => {
            let op = opes[0].string()?;
            match op.as_str() {
                "app" => {
                    let (f, arg) = (opes[1].clone(), opes[2].clone());
                    let mut fn_type = infer(ctx, f)?;
                    let arg_type = infer(ctx, arg)?;
                    let (_, ret_type) = type_var();
                    let new_fn_type = Type::Lambda(Box::new(arg_type), Box::new(ret_type.clone()));

                    println!("app: {} vs {}", fn_type, new_fn_type);
                    fn_type = unify(ctx, &fn_type, &new_fn_type)?;
                    println!("-> {}", fn_type);
                    Ok(prune(&fn_type.ret_type().unwrap()))
                }
                "lam" => {
                    let (arg, body) = (opes[1].string()?, opes[2].clone());
                    let (name, param_type) = type_var();
                    let mut ctx = ctx.copy();
                    ctx.env.insert(arg.clone(), param_type.clone());
                    ctx.references.insert(name, param_type.clone());
                    let body_ret = infer(&mut ctx, body)?;
                    ctx.env.insert(arg.clone(), param_type.clone());
                    let param = infer(&mut ctx, opes[1].clone())?;
                    ctx.env.insert(arg.clone(), param.clone());
                    Ok(Type::Lambda(Box::new(param), Box::new(body_ret)))
                }
                "let" => {
                    let (name, value, body) = (opes[1].string()?, opes[2].clone(), opes[3].clone());
                    let mut ctx = ctx.copy();
                    let value_type = infer(&mut ctx, value)?;
                    ctx.env.insert(name.to_string(), value_type.clone());
                    let res = infer(&mut ctx, body)?;
                    Ok(prune(&res))
                }
                _ => Err(anyhow::anyhow!("unknown operator: {}", op)),
            }
        }
        Sexp::Empty => Err(anyhow::anyhow!("empty expression")),
    };
    if let Ok(infer) = &ret {
        println!("do_infer {}: {}", &expr, infer);
    }
    ret
}

/// 単一化: 2つの型が一致するようななるべく小さな型代入を見つける
fn unify(ctx: &InferCtx, t: &Type, u: &Type) -> Result<Type> {
    let tp = prune(t);
    let up = prune(u);
    // if tp != up {
    //     return Err(anyhow::anyhow!("type mismatch: {:?} vs {:?}", &tp, &up))?;
    // }
    match (&tp, &up) {
        (Type::Lambda(t_arg, t_ret), Type::Lambda(u_arg, u_ret)) => {
            let arg = unify(ctx, t_arg, u_arg)?;
            let ret = unify(ctx, t_ret, u_ret)?;
            Ok(Type::Lambda(Box::new(arg), Box::new(ret)))
        }
        (Type::TypeVar(name, tvar), _) => {
            if tp != up {
                if occurs_in_type(&tp, &up) {
                    return Err(anyhow::anyhow!("recursive unification"));
                }
            }
            println!("typevar: {}={}", name, u);
            Ok(Type::TypeVar(name.clone(), Some(Box::new(u.clone()))))
        }
        (_, Type::TypeVar(..)) => unify(ctx, u, t),
        _ => Err(anyhow::anyhow!("unify error: {:?}, {:?}", t, u)),
    }
}

/// 型を具体的にする
fn prune(typ: &Type) -> Type {
    if let Type::TypeVar(_, tvar) = typ {
        if let Some(t) = tvar {
            return prune(t);
        }
    }
    typ.clone()
}

fn is_generic(ctx: &InferCtx, typ: &Type) -> bool {
    ctx.references
        .iter()
        .map(|(_, t)| occurs_in_type(t, typ))
        .any(|x| x)
}

/// typ 中に type_var が含まれているか
fn occurs_in_type(var: &Type, typ: &Type) -> bool {
    // prune(typ);
    if typ == var {
        return true;
    }
    if let Type::Lambda(arg_type, ret_type) = typ {
        let r1 = occurs_in_type(var, arg_type);
        let r2 = occurs_in_type(var, ret_type);
        return r1 || r2;
    } else {
        return false;
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use symbolic_expressions::parser::parse_str;

    use crate::{
        infer::{infer, InferCtx},
        types::{typ, Type},
    };

    fn should_infer(expr: &str, typ: Type) -> Result<()> {
        let mut ctx = InferCtx::new();
        let exp = parse_str(expr)?;
        let infer_typ = infer(&mut ctx, exp)?;
        println!("infer: {}", infer_typ);
        assert_eq!(infer_typ, typ);
        Ok(())
    }

    #[test]
    fn test_type_var() -> Result<()> {
        should_infer("true", Type::bool())
    }

    #[test]
    fn test_type_lambda() -> Result<()> {
        should_infer("(lam x 1)", typ("t1").to(Type::int()))
    }

    #[test]
    fn test_type_not() -> Result<()> {
        should_infer("(lam x (app not x))", Type::bool().to(Type::bool()))
    }
}
