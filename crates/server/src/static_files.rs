use axum::routing::MethodRouter;
use tower_http::services::ServeDir;

/// Serve the compiled Svelte SPA from `dist/ui/`.
///
/// In development, the Vite dev server handles the frontend directly
/// (with CORS enabled on the Rust server). In production, the built
/// SPA is served as static files from this path.
pub fn static_handler() -> MethodRouter {
    axum::routing::get_service(ServeDir::new("dist/ui").fallback(ServeDir::new("dist/ui")))
}
