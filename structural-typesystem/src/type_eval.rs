use crate::{
    type_env::TypeEnv,
    types::{Id, Type},
};
use anyhow::Result;
use symbolic_expressions::Sexp;

pub fn ensure_subtype(env: &mut TypeEnv, a: Id, b: Id) -> Result<()> {
    if !env.is_subtype(a, b)? {
        return Err(anyhow::anyhow!(
            "{} is not subtype of {}",
            env.type_name(a)?,
            env.type_name(b)?
        ));
    }
    Ok(())
}

fn eval_type_access(env: &mut TypeEnv, record_t: Sexp, key_t: Sexp) -> Result<Sexp> {
    let record_t = type_eval(env, record_t)?;
    let record_t_id = env.new_type(&record_t)?;
    let key_t = type_eval(env, key_t)?.string().cloned()?;
    let key_atom = key_t.trim_start_matches(":");
    let Type::Record { fields, .. } = env.alloc.get(record_t_id)? else {
                return Err(anyhow::anyhow!("{} is not record type", record_t));
            };
    let field_t_id = fields
        .get(key_atom)
        .ok_or_else(|| anyhow::anyhow!("{} is not found in {}", key_t, record_t))?;
    let field_t = env.type_name(*field_t_id)?;
    Ok(field_t)
}

pub fn type_eval(env: &mut TypeEnv, ty: Sexp) -> Result<Sexp> {
    log::debug!("type_eval: {}", ty);
    let res = match ty {
        t @ Sexp::String(_) => Ok(t),
        Sexp::List(list) if list[0].string()? == "[]" => {
            let (record_t, key_t) = (list[1].clone(), list[2].clone());
            eval_type_access(env, record_t, key_t)
        }
        t => Ok(t),
    }?;
    log::debug!("-> {}", res);
    Ok(res)
}
