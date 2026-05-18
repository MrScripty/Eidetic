mod ai_backends;
mod bible_graph_command;
mod bible_graph_edge_store;
mod bible_graph_field_store;
mod bible_graph_schema;
mod bible_graph_snapshot_store;
mod bible_graph_store;
mod bible_graph_value_store;
mod diffusion;
mod embeddings;
pub mod error;
mod export;
mod history_store;
mod object_field_command;
mod persistence;
mod prompt_format;
mod revision_projection;
mod routes;
mod script_document_command;
mod script_store_codec;
mod script_store_schema;
mod script_store;
mod sqlite;
mod state;
mod static_files;
mod story_arc_command;
mod timeline_command;
mod timeline_command_history;
mod timeline_command_history_codec;
mod timeline_node_delete_history;
mod validation;
mod vector_store;
mod ws;
mod ydoc;

use std::net::SocketAddr;

use axum::Router;
use axum::http::{Method, header};
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

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin, _request_parts| {
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
