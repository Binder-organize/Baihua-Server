use crate::ServerState;
use crate::console::CommandType;
use axum::Router;
use axum::routing::{get, post};
use tracing::{error, info};

pub async fn server(mut command_rx: tokio::sync::mpsc::Receiver<CommandType>, state: ServerState) {
    info!("The server is started.");

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/", post(|| async { "Hello, World!" }));

    // read IP addresses.
    let address = format!("{}:{}", state.config.server.host, state.config.server.port);
    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();

    // Start the server.
    match axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            loop {
                tokio::select! {
                    cmd = command_rx.recv() => {
                        match cmd {
                            Some(CommandType::Shutdown) => {
                                break;
                            }
                            None => {}
                            _ => {}
                        }
                    }
                }
            }
        })
        .await
    {
        Ok(_) => {}
        Err(e) => {
            error!("Server startup failed: {}.", e);
        }
    }
}
