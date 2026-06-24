// Baihua Server, by Gavin Zheng on January 1, 2026.

mod authenticate;
mod common;
mod console;
mod greet;
mod health;
mod infrastructure;
mod middleware;
mod server;
mod user;

use anyhow::Result;
use infrastructure::config::AppConfigure;
use infrastructure::environment::Environment;
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
    pub configure: AppConfigure,
    pub pool: PgPool,
    pub jwt_secret: String,
    pub environment: Environment,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Determine the production/development environment.
    let env = Environment::from_env();

    if env.is_development() {
        let _ = dotenvy::dotenv();
    }

    println!(
        "Baihua Server - v0.1.0 ({})",
        if env.is_production() {
            "production"
        } else {
            "development"
        }
    );

    // Initialize the server.
    let (state, log_guard) = match initialize::initialize(env).await {
        Ok((server_state, guard)) => (server_state, guard),
        Err(error) => {
            eprintln!("Server initialization failed: {}.", error);
            return Err(error);
        }
    };

    info!(
        "Server address: {}:{}.",
        state.configure.server.host, state.configure.server.port
    );

    let (command_tx, command_rx) = tokio::sync::mpsc::channel::<console::CommandType>(32);
    let server_handle = tokio::spawn(server::server(command_rx, state.clone()));

    let shutdown_reason = if state.environment.is_development() {
        let console_handle = tokio::spawn(console::console(command_tx));

        tokio::select! {
            reason = console_handle => {
                match reason {
                    Ok(()) => "Console requested shutdown.",
                    Err(error) => {
                        error!("Console task failed: {}", error);
                        "Console task failed"
                    }
                }
            }
            reason = server_handle => {
                match reason {
                    Ok(Ok(())) => "Server completed successfully.",
                    Ok(Err(error)) => {
                        error!("Server task failed: {}.", error);
                        "Server task failed."
                    }
                    Err(error) => {
                        error!("Server task panicked: {}.", error);
                        "Server task panicked."
                    }
                }
            }
            _ = shutdown_signal() => {
                "Received shutdown signal."
            }
        }
    } else {
        info!("Console disabled in production mode.");
        let _command_tx = command_tx;

        tokio::select! {
            reason = server_handle => {
                match reason {
                    Ok(Ok(())) => "Server completed successfully.",
                    Ok(Err(error)) => {
                        error!("Server task failed: {}.", error);
                        "Server task failed."
                    }
                    Err(error) => {
                        error!("Server task panicked: {}.", error);
                        "Server task panicked."
                    }
                }
            }
            _ = shutdown_signal() => {
                "Received shutdown signal."
            }
        }
    };

    info!("Shutting down: {}", shutdown_reason);

    info!("Saving log.");
    drop(log_guard);
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    println!("Goodbye!");

    Ok(())
}

// Waits for either SIGINT or SIGTERM (Unix) or Ctrl-C (Windows).
pub(crate) async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut sigint = signal(SignalKind::interrupt()).expect("Failed to set up SIGINT handler.");
        let mut sigterm =
            signal(SignalKind::terminate()).expect("Failed to set up SIGTERM handler.");

        tokio::select! {
            _ = sigint.recv() => {}
            _ = sigterm.recv() => {}
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl-C.");
    }
}
