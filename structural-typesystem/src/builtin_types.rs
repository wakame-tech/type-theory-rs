use crate::{type_alloc::TypeAlloc, type_env::TypeEnv};
use anyhow::Result;

/// register builtin types
pub fn register_builtin_types(env: &mut TypeEnv, alloc: &mut TypeAlloc) -> Result<()> {
    let any = alloc.new_primitive("any");
    env.add("any", any);

    let int = alloc.new_primitive("int");
    env.add("int", int);
    env.subtype(int, any);

    let bool = alloc.new_primitive("bool");
    env.add("bool", bool);
    env.subtype(bool, any);

    env.debug(alloc)?;
    Ok(())
}
