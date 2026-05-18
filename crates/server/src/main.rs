mod ai_backends;
mod diffusion;
mod embeddings;
pub mod error;
mod export;
mod history_store;
mod persistence;
mod prompt_format;
mod routes;
mod state;
mod static_files;
mod validation;
mod vector_store;
mod ws;
mod ydoc;

use std::net::SocketAddr;

use axum::Router;
use axum::http::{header, Method};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let app_state = state::AppState::new().await;

    let cors = CorsLayer::new().allow_origin(AllowOrigin::predicate(|origin, _request_parts| {
        origin
            .to_str()
            .map(validation::is_allowed_local_origin)
            .unwrap_or(false)
    }))
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers([header::CONTENT_TYPE]);

    let app = Router::new()
        .nest("/api", routes::api_router())
        .route("/ws", axum::routing::get(ws::ws_handler))
        .fallback_service(static_files::static_handler())
        .layer(cors)
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
