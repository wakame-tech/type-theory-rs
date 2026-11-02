use std::collections::BTreeSet;

use crate::{
    type_env::TypeEnv,
    types::{Id, Type, GETTER_TYPE_KEYWORD},
};
use anyhow::Result;
use symbolic_expressions::Sexp;

pub fn ensure_subtype(env: &mut TypeEnv, a: Id, b: Id) -> Result<()> {
    if !env.is_subtype(a, b)? {
        return Err(anyhow::anyhow!(
            "{} is not subtype of {}",
            env.type_name(a)?,
            env.type_name(b)?,
        ));
    }
    Ok(())
}

fn eval_type_access(env: &mut TypeEnv, record: Id, key: Id) -> Result<Id> {
    let record = type_eval(env, record)?;
    let Type::Record { fields, .. } = env.alloc.get(record)? else {
        return Err(anyhow::anyhow!("{} is not record type", record));
    };
    let key = type_eval(env, key)?;
    let Sexp::String(atom) = env.type_name(key)? else {
        return Err(anyhow::anyhow!(
            "{} #{} is not atom type",
            env.type_name(key)?,
            key
        ));
    };
    let key = atom.trim_start_matches(':');
    fields.get(key).copied().ok_or_else(|| {
        anyhow::anyhow!(
            "key :{} not found in record {}",
            key,
            env.type_name(record).unwrap()
        )
    })
}

pub fn type_eval(env: &mut TypeEnv, id: Id) -> Result<Id> {
    match env.alloc.get(id)? {
        Type::Variable {
            instance: Some(instance),
            ..
        } => type_eval(env, instance),
        // flatten union type
        Type::Union { types, .. } => {
            let mut flat = BTreeSet::new();
            for t in types {
                let t = type_eval(env, t)?;
                match env.alloc.get(t)? {
                    Type::Union { types, .. } => flat.extend(types),
                    _ => {
                        flat.insert(t);
                    }
                }
            }
            env.new_union(flat)
        }
        Type::Container { id: con, elements } if matches!(env.alloc.get(con)?, Type::Primitive { name, .. } if name == GETTER_TYPE_KEYWORD) => {
            eval_type_access(env, elements[0], elements[1])
        }
        _ => Ok(id),
    }
}

#[cfg(test)]
mod tests {
    use crate::{type_env::TypeEnv, type_eval::type_eval};
    use anyhow::Result;

    fn assert_type_eval(t: &str, s: &str) -> Result<()> {
        let mut env = TypeEnv::default();
        let t = env.new_type_str(t)?;
        let s = env.new_type_str(s)?;
        assert_eq!(
            type_eval(&mut env, t)?,
            s,
            "{} != {}",
            env.type_name(t)?,
            env.type_name(s)?
        );
        Ok(())
    }

    #[test]
    fn test_type_eval() -> Result<()> {
        assert_type_eval("([] (record (a : int)) :a)", "int")?;
        assert_type_eval("(| (| 1) 2)", "(| 1 2)")?;
        Ok(())
    }
}
