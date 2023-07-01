use crate::type_env::TypeEnv;
use symbolic_expressions::parser::parse_str;

impl Default for TypeEnv {
    fn default() -> Self {
        let mut env = TypeEnv::new();
        let any = env.new_type(&parse_str("any").unwrap()).unwrap();
        let int = env.new_type(&parse_str("int").unwrap()).unwrap();
        let bool = env.new_type(&parse_str("bool").unwrap()).unwrap();
        env.new_subtype(int, any);
        env.new_subtype(bool, any);
        env
    }
}
