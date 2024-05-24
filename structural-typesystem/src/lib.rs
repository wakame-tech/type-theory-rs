pub mod infer;
pub mod issuer;
pub mod subtyping;
pub mod type_alloc;
pub mod type_check;
pub mod type_env;
pub mod type_eval;
pub mod types;

#[cfg(test)]
pub(crate) mod tests {
    use simple_logger::SimpleLogger;
    use std::sync::Once;

    static INIT: Once = Once::new();

    pub fn setup() {
        INIT.call_once(|| {
            SimpleLogger::new()
                .without_timestamps()
                .with_level(log::LevelFilter::Debug)
                .init()
                .unwrap();
        });
    }
}
