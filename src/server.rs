use crate::ServerState;
use crate::console::CommandType;
use crate::greet::greet;
use crate::middleware;
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
        .nest("/api/v1", api_v1(state.clone()))
        .layer(axum::middleware::from_fn(middleware::tracing::tracing)) // Track requests.
        .layer(axum::middleware::from_fn(middleware::error::panic)) // Handle panic.
        .layer(axum::middleware::from_fn(middleware::error::not_found)); // Handle 404.

    // read IP addresses.
    let address = format!(
        "{}:{}",
        state.configure.server.host, state.configure.server.port
    );
    let listener = tokio::net::TcpListener::bind(&address).await?;

    // Setup graceful shutdown handler
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
            _ = tokio::signal::ctrl_c() => {
                info!("Received Ctrl+C signal.");
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

fn api_v1(state: Arc<ServerState>) -> Router {
    Router::new().nest("/user", crate::user::router(state))
}
