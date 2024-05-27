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
    use std::sync::Once;

    static INIT: Once = Once::new();

    pub fn setup() {
        INIT.call_once(|| {
            tracing_subscriber::fmt()
                .with_test_writer()
                .without_time()
                .with_max_level(tracing::Level::DEBUG)
                .with_line_number(true)
                .init();
        });
    }
}
