pub mod cargo_env {
    pub const CARGO_PKG_NAME: &'static str = env!("CARGO_PKG_NAME");
}

pub mod common {
    pub const MAX_PARALLEL_REQUESTS: usize = 64;
}
