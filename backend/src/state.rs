use std::sync::{Arc, atomic::AtomicI32};

#[derive(Clone)]
pub struct AppState {
    pub count: Arc<AtomicI32>,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            count: Arc::new(AtomicI32::new(0)),
        }
    }
}
