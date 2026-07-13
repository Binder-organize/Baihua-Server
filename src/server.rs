use crate::ServerState;
use crate::console::CommandType;
use crate::greet::greet;
use crate::health::health_check;
use crate::middleware;
use crate::websocket;
use axum::Router;
use axum::routing::get;
use std::sync::Arc;
use tracing::info;

pub async fn server(
    mut command_rx: tokio::sync::mpsc::Receiver<CommandType>,
    state: ServerState,
) -> Result<(), anyhow::Error> {
    info!("The server is started.");

    let state = Arc::new(state);

    let app = Router::new()
        .route("/greet", get(greet))
        .route("/health", get(health_check))
        .route("/websocket", get(websocket::handler::ws_handler))
        .nest("/api/v1", api_v1(state.clone()))
        .layer(axum::middleware::from_fn(middleware::tracing::tracing))
        .layer(axum::middleware::from_fn(middleware::error::panic))
        .layer(axum::middleware::from_fn(middleware::error::not_found))
        .with_state(state.clone());

    // read IP addresses.
    let address = format!(
        "{}:{}",
        state.configure.server.host, state.configure.server.port
    );
    let listener = tokio::net::TcpListener::bind(&address).await?;

    // Setup graceful shutdown handler (SIGINT + SIGTERM)
    let shutdown_handler = async move {
        tokio::select! {
            command = command_rx.recv() => {
                match command {
                    Some(CommandType::Shutdown) => {
                        info!("Received shutdown command from console.");
                    }
                    None => {
                        info!("Command channel closed.");
                    }
                }
            }
            _ = crate::shutdown_signal() => {
                info!("Received OS shutdown signal.");
            }
        }
    };

    // Start the server with graceful shutdown and proper error handling
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_handler)
        .await?;

    info!("Server shutdown completed successfully");
    Ok(())
}

fn api_v1(state: Arc<ServerState>) -> Router<Arc<ServerState>> {
    Router::new()
        .nest("/user", crate::user::router(state.clone()))
        .nest("/chat", crate::chat::router(state))
}
