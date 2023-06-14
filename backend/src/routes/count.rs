use crate::state::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Json,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};
use client::{CountRequest, CountResponse, Direction};

#[derive(Debug)]
enum ServerError {
    MutexError,
    SerialisationError,
}

pub async fn get_count(Extension(state): Extension<AppState>) -> impl IntoResponse {
    match try_get_count(&state) {
        Ok(json) => Ok(json),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

fn try_get_count(state: &AppState) -> Result<String, ServerError> {
    match state.count.try_lock() {
        Ok(mutex) => {
            let count = *mutex;
            match serde_json::to_string(&CountResponse { count }) {
                Ok(j) => Ok(j),
                Err(_) => Err(ServerError::SerialisationError),
            }
        }
        Err(_) => Err(ServerError::MutexError),
    }
}

pub async fn post_count(
    Extension(state): Extension<AppState>,
    payload: Json<CountRequest>,
) -> impl IntoResponse {
    let request: CountRequest = payload.0;
    match try_alter_count(&state, request) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn try_alter_count(state: &AppState, request: CountRequest) -> Result<(), ServerError> {
    match state.count.try_lock() {
        Ok(ref mut mutex) => Ok(match request.direction {
            Direction::Increment => **mutex += 1,
            Direction::Decrement => **mutex -= 1,
        }),
        Err(_) => Err(ServerError::MutexError),
    }
}

pub async fn ws_handler(ws: WebSocketUpgrade, Extension(state): Extension<AppState>) -> Response {
    log::info!("client connected");
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    if let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(_) => {
                    let response_json = serde_json::to_string(&CountResponse {
                        count: *state.count.lock().unwrap(),
                    });
                    match response_json {
                        Ok(j) => {
                            // send message
                            if socket.send(Message::Text(j)).await.is_err() {
                                log::error!("client disconnected during transfer");
                            }
                        }
                        Err(_) => log::error!("abject failure to build JSON"),
                    }
                }
                Message::Binary(_) => {
                    log::error!("client sent binary data");
                }
                Message::Ping(_) => {
                    log::info!("socket ping");
                }
                Message::Pong(_) => {
                    log::info!("socket pong");
                }
                Message::Close(_) => {
                    log::info!("client disconnected");
                    return;
                }
            }
        } else {
            log::info!("client disconnected");
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn try_get_count_returns_json_count_response() {
        let state = AppState::new();
        let resp = try_get_count(&state).unwrap();
        assert!(resp == r#"{"count":0}"#);
    }
    
    #[test]
    fn try_alter_count_increments_then_decrements_state() {
        let state = AppState::new();
        try_alter_count(&state, CountRequest { direction: Direction::Increment}).expect("failed to incremend state");
        assert!(*state.count.lock().unwrap() == 1);        
        try_alter_count(&state, CountRequest { direction: Direction::Decrement }).expect("failed to decrement state");
        assert!(*state.count.lock().unwrap() == 0);
    }
}