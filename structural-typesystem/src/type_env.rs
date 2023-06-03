use std::{collections::HashMap, fmt::Debug};

use crate::{type_alloc::TypeAlloc, types::Id};
use anyhow::Result;
use log::debug;

/// [TypeEnv] will be created per each [Expr::FnDef]
#[derive(Debug, Clone)]
pub struct TypeEnv {
    pub id_map: HashMap<String, Id>,
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self {
            id_map: HashMap::new(),
        }
    }
}

impl TypeEnv {
    pub fn register(&mut self, name: &str, type_id: Id) {
        self.id_map.insert(name.to_string(), type_id);
    }

    pub fn get_id(&self, name: &str) -> Option<Id> {
        self.id_map.get(name).cloned()
    }
}

/// ### builtin functions
/// - `not`
/// - `id`
/// - `zero?`
/// - `succ`
pub fn register_buildin_fns(env: &mut TypeEnv, alloc: &mut TypeAlloc) -> Result<()> {
    let int = alloc.from("int")?;
    let bool = alloc.from("bool")?;
    let a = alloc.new_variable();

    env.register("true", bool);
    env.register("false", bool);

    let not_type = alloc.new_function(bool, bool);
    env.register("not", not_type);

    let id_type = alloc.new_function(a, a);
    env.register("id", id_type);

    let zero_type = alloc.new_function(int, bool);
    env.register("zero?", zero_type);

    let succ_type = alloc.new_function(int, int);
    env.register("succ", succ_type);

    Ok(())
}

pub fn setup_type_env() -> Result<(TypeEnv, TypeAlloc)> {
    let mut alloc = TypeAlloc::default();
    let mut env = TypeEnv::default();
    register_buildin_fns(&mut env, &mut alloc)?;
    Ok((env, alloc))
}

#[cfg(test)]
mod test {
    use crate::type_env::setup_type_env;
    use log::LevelFilter;
    use std::io::Write;
    use std::sync::Once;

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

    #[test]
    fn test_int() {
        setup();
        let (mut env, mut alloc) = setup_type_env().unwrap();
        let int = alloc.from("int").unwrap();
        assert_eq!(
            alloc.as_string(int, &mut Default::default()).unwrap(),
            "int"
        );
    }
}
