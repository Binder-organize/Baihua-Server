use crate::ServerState;
use crate::console::CommandType;
use crate::greet::greet;
use axum::Router;
use axum::routing::get;
use std::sync::Arc;
use tracing::{error, info};

pub async fn server(mut command_rx: tokio::sync::mpsc::Receiver<CommandType>, state: ServerState) {
    info!("The server is started.");

    let state = Arc::new(state);

    let app = Router::new()
        .route("/greet", get(greet))
        .nest("/api/v1", api_v1(state.clone()));

    // read IP addresses.
    let address = format!(
        "{}:{}",
        state.configure.server.host, state.configure.server.port
    );
    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();

    // todo need to optimize.
    // Start the server.
    match axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            loop {
                tokio::select! {
                    command = command_rx.recv() => {
                        match command {
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

fn api_v1(state: Arc<ServerState>) -> Router {
    Router::new().nest("/user", crate::user::router(state))
}
