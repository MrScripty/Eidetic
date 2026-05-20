use std::net::SocketAddr;

use axum::Router;
use axum::http::{Method, Uri, header};
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
                .map(is_allowed_local_origin)
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

fn is_allowed_local_origin(origin: &str) -> bool {
    let Ok(uri) = Uri::try_from(origin) else {
        return false;
    };

    let Some(scheme) = uri.scheme_str() else {
        return false;
    };
    if scheme != "http" && scheme != "https" {
        return false;
    }

    matches!(
        uri.host(),
        Some("127.0.0.1" | "localhost" | "[::1]" | "::1")
    )
}

#[cfg(test)]
mod tests {
    use super::is_allowed_local_origin;

    #[test]
    fn local_origin_policy_allows_loopback_hosts() {
        assert!(is_allowed_local_origin("http://127.0.0.1:5173"));
        assert!(is_allowed_local_origin("http://localhost:3000"));
        assert!(is_allowed_local_origin("https://localhost:3000"));
    }

    #[test]
    fn local_origin_policy_rejects_non_loopback_hosts() {
        assert!(!is_allowed_local_origin("https://example.com"));
        assert!(!is_allowed_local_origin("file:///tmp/index.html"));
    }
}
