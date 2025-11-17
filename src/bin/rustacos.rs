use clap::Parser;
use axum::http::Method;
use tower_http::cors::{Any, CorsLayer};
use tokio::net::TcpListener;

#[derive(Parser, Debug)]
#[command(name = "rustacos")]
#[command(about = "Rustacos server (app-bootstrap)")]
struct Args {
    #[arg(short, long, default_value_t = 8848)]
    port: u16,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let app = app_bootstrap::build_app()
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PUT, Method::OPTIONS])
                .allow_headers(Any),
        );
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], args.port));
    tracing::info!("rustacos listening on {}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


