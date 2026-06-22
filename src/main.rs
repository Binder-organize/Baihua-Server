// Baihua Server, by Gavin Zheng on January 1, 2026.

mod authenticate;
mod common;
mod console;
mod greet;
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

    let console_handle = tokio::spawn(console::console(command_tx));
    let server_handle = tokio::spawn(server::server(command_rx, state.clone()));

    let shutdown_reason = tokio::select! {
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
        _ = tokio::signal::ctrl_c() => {
            "Received interrupt signal (Ctrl-C)."
        }
    };

    info!("Shutting down: {}", shutdown_reason);

    info!("Saving log.");
    drop(log_guard);
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    println!("Goodbye!");

    Ok(())
}
