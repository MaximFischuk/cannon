pub mod error;
pub mod timeunit;

#[macro_export]
macro_rules! now {
    () => {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::default())
            .as_millis()
    };
}
