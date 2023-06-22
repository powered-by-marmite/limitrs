use std::{sync::atomic::Ordering, str::FromStr};
use std::borrow::Cow;
use std::ops::ControlFlow;
use crate::state::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade, CloseFrame},
        Path,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};
use client::{CountRequest, CountResponse, Direction};
use futures::{sink::SinkExt, stream::StreamExt};

#[derive(Debug, PartialEq)]
enum ServerError {
    MaximumValueError,
    SerialisationError,
}

pub async fn get_count(Extension(state): Extension<AppState>) -> impl IntoResponse {
    match try_get_count(&state) {
        Ok(json) => Ok(json),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

fn try_get_count(state: &AppState) -> Result<String, ServerError> {
    let count = state.count.load(Ordering::Relaxed);
    match serde_json::to_string(&CountResponse { count }) {
        Ok(j) => Ok(j),
        Err(_) => Err(ServerError::SerialisationError),
    }
}

pub async fn post_count(
    Extension(state): Extension<AppState>,
    Path(direction): Path<String>,
) -> impl IntoResponse {
    if let Ok(request) = CountRequest::from_str(&direction) {
        match try_alter_count(&state, request) {
            Ok(_) => StatusCode::OK,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    } else {
        StatusCode::BAD_REQUEST
    }
}

fn try_alter_count(state: &AppState, request: CountRequest) -> Result<(), ServerError> {
    match request.direction {
        Direction::Increment => {
            if state.count.load(Ordering::Relaxed) == i32::MAX {
                return Err(ServerError::MaximumValueError);
            }
            state.count.fetch_add(1, Ordering::Relaxed)
        }
        Direction::Decrement => state.count.fetch_sub(1, Ordering::Relaxed),
    };
    Ok(())
}

pub async fn ws_handler(ws: WebSocketUpgrade, Extension(state): Extension<AppState>) -> Response {
    log::info!("client connected");
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket<>(mut socket: WebSocket, state: AppState) {
    // send a ping to ensure the connection upgrade succeeded
    if socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
        log::trace!("sent startup ping");
    } else {
        log::error!("failed to send startup ping");
        return;
    }

    // we need to both send and receive messages
    let (mut sender, mut receiver) = socket.split();

    let _send_task = tokio::spawn(async move {
        let mut latest_count = state.count.load(Ordering::Relaxed);
        
        // on connection, send initial state
        let response_json = serde_json::to_string(&CountResponse {
            count: latest_count,
        });
        match response_json {
            Ok(j) => {
                // send message
                if sender
                    .send(Message::Text(j))
                    .await
                    .is_err() {
                    log::error!("client disconnected during transfer");
                }
            }
            Err(_) => log::error!("abject failure to build JSON"),
        }

        // infinite loop to dispatch state changes to socket
        loop {
            let count = state.count.load(Ordering::Relaxed);
            if count != latest_count {
                latest_count = count;

                let response_json = serde_json::to_string(&CountResponse {
                    count,
                });
                match response_json {
                    Ok(j) => {
                        // send message
                        if sender
                            .send(Message::Text(j))
                            .await
                            .is_err() {
                            log::error!("client disconnected during transfer");
                            break;
                        }
                    }
                    Err(_) => log::error!("abject failure to build JSON"),
                }
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        sender.send(Message::Close(Some(CloseFrame {
            code: axum::extract::ws::close_code::NORMAL,
            reason: Cow::from("hanging up"),
        }))).await
    });

    // spawn a task which receives messages from the socket
    let _recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if process_message(msg).is_break() {
                break;
            }
        }
    });

}

fn process_message(msg: Message) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(_) => {
            log::info!("client sent text data");
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
            return ControlFlow::Break(());
        }
    }
    ControlFlow::Continue(())
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
        assert!(state.count.load(Ordering::Acquire) == 1);        
        try_alter_count(&state, CountRequest { direction: Direction::Decrement }).expect("failed to decrement state");
        assert!(state.count.load(Ordering::Acquire) == 0);
    }

    #[test]
    fn try_alter_count_fails_to_increment_at_maxiumum_value() {
        let state = AppState::new();
        state.count.swap(i32::MAX, Ordering::Acquire);
        let result = try_alter_count(&state, CountRequest { direction: Direction::Increment});
        let expected = Err(ServerError::MaximumValueError);
        assert_eq!(result, expected);
    }

    #[test]
    fn try_alter_count_decrements_maximum_value() {
        let state = AppState::new();
        state.count.swap(i32::MAX, Ordering::Acquire);
        try_alter_count(&state, CountRequest { direction: Direction::Decrement }).expect("failed to decrement state");
        assert_eq!(state.count.load(Ordering::Acquire), i32::MAX - 1);
    }
}
