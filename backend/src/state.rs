use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub count: Arc<Mutex<i32>>,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            count: Arc::new(Mutex::new(0)),
        }
    }
}
