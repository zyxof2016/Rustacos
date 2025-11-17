use axum::http::Method;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    let app = app_bootstrap::build_app()
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PUT, Method::OPTIONS])
            .allow_headers(Any)
        );
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8850));
    tracing_subscriber::fmt::init();
    tracing::info!("nextapp listening on {}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


