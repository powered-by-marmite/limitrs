use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Json, State,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Extension, Router,
};
use clap::Parser;
use client::{CountRequest, CountResponse, Direction};
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

const LOGGING_VARIABLE: &str = "RUST_LOG";

// Setup the command line interface with clap.
#[derive(Parser, Debug)]
#[clap(name = "server", about = "A server for our wasm project!")]
struct Opt {
    /// set the log level
    #[clap(short = 'l', long = "log-level", default_value = "debug")]
    log_level: String,

    /// set the listen addr
    #[clap(short = 'a', long = "addr", default_value = "::1")]
    addr: String,

    /// set the listen port
    #[clap(short = 'p', long = "port", default_value = "8080")]
    port: u16,

    /// set the directory where static files are to be found
    #[clap(long = "static-dir", default_value = "./dist")]
    static_dir: String,
}

#[derive(Clone)]
struct AppState {
    pub count: Arc<Mutex<i32>>,
}

impl AppState {
    fn new() -> AppState {
        AppState {
            count: Arc::new(Mutex::new(0)),
        }
    }
}

#[tokio::main]
async fn main() {
    let opt = Opt::parse();

    // set up logging
    match std::env::var(LOGGING_VARIABLE) {
        Ok(_) => (),
        Err(_) => std::env::set_var(
            LOGGING_VARIABLE,
            format!("{},hyper=info,mio=info", opt.log_level),
        ),
    }
    // log to the console
    tracing_subscriber::fmt::init();

    let state = AppState::new();
    let also_state = AppState::new();

    let app = Router::new()
        .route(
            "/api/count",
            get(get_count).post(post_count).with_state(state),
        )
        .route("/ws/count", get(ws_handler))
        .merge(axum_extra::routing::SpaRouter::new(
            "/assets",
            opt.static_dir,
        ))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .layer(Extension(also_state));

    let sock_addr = SocketAddr::from((
        IpAddr::from_str(opt.addr.as_str()).unwrap_or(IpAddr::V6(Ipv6Addr::LOCALHOST)),
        opt.port,
    ));

    log::info!("listening on http://{}", sock_addr);

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .await
        .expect("Unable to start server");
}

async fn get_count(State(state): State<AppState>) -> impl IntoResponse {
    let response_json = serde_json::to_string(&CountResponse {
        count: *state.count.lock().unwrap(),
    });
    match response_json {
        Ok(j) => Ok(format!("{}", j)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn post_count(State(state): State<AppState>, payload: Json<CountRequest>) {
    let payload: CountRequest = payload.0;
    match payload.direction {
        Direction::Increment => {
            let mut s = state.count.lock().unwrap();
            *s += 1;
        }
        Direction::Decrement => {
            let mut s = state.count.lock().unwrap();
            *s -= 1;
        }
    };
}

async fn ws_handler(ws: WebSocketUpgrade, Extension(state): Extension<AppState>) -> Response {
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
                        Err(_) => log::error!(""),
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
