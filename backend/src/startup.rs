use crate::{
    routes::count::{get_count, post_count, ws_handler},
    routes::health_check::health_check,
    state::AppState,
};
use axum::{http::StatusCode, routing::get, Extension, Router};
use std::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

async fn fallback() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not Found")
}

pub async fn run(listener: TcpListener, static_dir: String) {
    let app = build_router(static_dir);
    
    axum::Server::from_tcp(listener).expect("failed to bind to socket address")
        .serve(app.into_make_service())
        .await
        .expect("Unable to start server");
}

fn build_router(static_dir: String) -> Router {
    let state = AppState::new();

    Router::new()
        .route("/health_check", get(health_check))
        .route(
            "/api/count",
            get(get_count).post(post_count),
        )
        .route("/ws/count", get(ws_handler))
        // frontend serving: static assets for the Single-Page Application
        .merge(axum_extra::routing::SpaRouter::new("/assets", static_dir))
        .fallback(fallback)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .layer(Extension(state))
}
