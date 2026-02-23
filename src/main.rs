// Baihua Server, by Gavin Zheng on January 1, 2026.

mod common;
mod console;
mod greet;
mod infrastructure;
mod middleware;
mod server;
mod user;

use anyhow::Result;
use infrastructure::config::AppConfigure;
use infrastructure::initialize;
use sqlx::PgPool;
use std::path::PathBuf;
use tracing::{error, info};

#[derive(Debug, Clone)]
pub struct Directory {
    pub app: PathBuf,
    pub log: PathBuf,
}

#[derive(Clone)]
pub struct ServerState {
    configure: AppConfigure,
    pool: PgPool,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Baihua Server - v0.1.0");

    // Initialize the application environment.
    let (state, log_guard) = match initialize::initialize().await {
        Ok((server_state, guard)) => (server_state, guard),
        Err(error) => {
            eprintln!("Server initialization failed: {}.", error);
            return Err(error);
        }
    };

    // Record startup information.
    info!(
        "Server address: {}:{}.",
        state.configure.server.host, state.configure.server.port
    );

    let (command_tx, command_rx) = tokio::sync::mpsc::channel::<console::CommandType>(32);
    let console_handle = tokio::spawn(console::console(command_tx));
    let server_handle = tokio::spawn(server::server(command_rx, state));

    // Wait for the server or console to end.
    let shutdown_reason = tokio::select! {
        reason = console_handle => {
            match reason {
                Ok(()) => "Console requested shutdown.",
                Err(e) => {
                    error!("Console task failed: {}", e);
                    "Console task failed"
                }
            }
        }
        reason = server_handle => {
            match reason {
                Ok(Ok(())) => "Server completed successfully.",
                Ok(Err(e)) => {
                    error!("Server task failed: {}.", e);
                    "Server task failed."
                }
                Err(e) => {
                    error!("Server task panicked: {}.", e);
                    "Server task panicked."
                }
            }
        }
        _ = tokio::signal::ctrl_c() => {
            "Received interrupt signal (Ctrl-C)."
        }
    };

    info!("Shutting down: {}", shutdown_reason);
    info!("Saving log.");

    drop(log_guard);

    // Make sure the logs are saved.
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    println!("Goodbye!");
    Ok(())
}
