use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CountRequest {
    pub direction: Direction,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CountResponse {
    pub count: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Direction {
    Increment,
    Decrement,
}
