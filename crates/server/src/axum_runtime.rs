use std::net::SocketAddr;

use axum::Router;
use axum::http::{Method, header};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::state::AppState;

pub fn default_server_addr() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], 3000))
}

pub fn build_router(app_state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin, _request_parts| {
            origin
                .to_str()
                .map(crate::validation::is_allowed_local_origin)
                .unwrap_or(false)
        }))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE]);

    Router::new()
        .nest("/api", crate::routes::api_router())
        .route("/ws", axum::routing::get(crate::ws::ws_handler))
        .fallback_service(crate::static_files::static_handler())
        .layer(cors)
        .with_state(app_state)
}

pub async fn serve_default() -> Result<(), Box<dyn std::error::Error>> {
    serve(default_server_addr()).await
}

pub async fn serve(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let app_state = AppState::new().await;
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!("failed to bind {addr}: {error}");
            return Err(error.into());
        }
    };

    tracing::info!("listening on {addr}");
    axum::serve(listener, build_router(app_state)).await?;
    Ok(())
}
