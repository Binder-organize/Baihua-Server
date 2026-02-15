use crate::ServerState;
use crate::console::CommandType;
use crate::greet::greet;
use axum::Router;
use axum::routing::get;
use tracing::{error, info};

pub async fn server(mut command_rx: tokio::sync::mpsc::Receiver<CommandType>, state: ServerState) {
    info!("The server is started.");

    // API v1
    let api_v1 = Router::new().nest("/user", crate::user::router());

    let app = Router::new()
        .route("/greet", get(greet))
        .nest("/api/v1/", api_v1);

    // read IP addresses.
    let address = format!("{}:{}", state.config.server.host, state.config.server.port);
    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();

    // todo need to optimize.
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
