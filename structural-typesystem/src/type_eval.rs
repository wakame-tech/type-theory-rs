use std::collections::BTreeSet;

use crate::{
    type_env::TypeEnv,
    types::{Id, Type, GETTER_TYPE_KEYWORD, UNION_TYPE_KEYWORD},
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
    let t = env.type_name(id)?;
    match t {
        Sexp::List(list) if list[0].is_string() && list[0].string()? == GETTER_TYPE_KEYWORD => {
            let (record, key) = (env.new_type(&list[1])?, env.new_type(&list[2])?);
            eval_type_access(env, record, key)
        }
        Sexp::List(list) if list[0].is_string() && list[0].string()? == UNION_TYPE_KEYWORD => {
            // flatten union type
            let mut types = list[1..]
                .iter()
                .map(|t| {
                    let id = env.get(t)?;
                    let t = type_eval(env, id)?;
                    let inner = if let Type::Union { types, .. } = env.alloc.get(t)? {
                        types
                    } else {
                        BTreeSet::from_iter(vec![t])
                    };
                    Ok(inner)
                })
                .collect::<Result<BTreeSet<_>>>()?
                .into_iter()
                .flatten()
                .map(|t| env.type_name(t))
                .collect::<Result<Vec<_>>>()?;
            types.insert(0, Sexp::String(UNION_TYPE_KEYWORD.to_string()));
            env.new_type(&Sexp::List(types))
        }
        t => env.new_type(&t),
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
