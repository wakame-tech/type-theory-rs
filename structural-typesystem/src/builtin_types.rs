use crate::{type_alloc::TypeAlloc, type_env::TypeEnv};

impl Default for TypeAlloc {
    fn default() -> Self {
        let mut alloc = TypeAlloc::new();
        alloc.new_primitive("any");
        alloc.new_primitive("int");
        alloc.new_primitive("bool");
        alloc
    }
}

impl Default for TypeEnv {
    fn default() -> Self {
        let alloc = TypeAlloc::default();
        let mut env = TypeEnv::new(alloc);
        let any = env.get("any").unwrap();
        let int = env.get("int").unwrap();
        let bool = env.get("bool").unwrap();
        env.add("any", any);
        env.add("int", int);
        env.subtype(int, any);
        env.add("bool", bool);
        env.subtype(bool, any);

        env
    }
}
