use crate::infrastructure::config::DatabaseConfiguration;
use crate::infrastructure::environment::Environment;
use anyhow::{Context, Result};
use sqlx::{Pool, Postgres};

pub fn build_connection_url(environment: Environment) -> Result<String> {
    let user = environment.require_variable("POSTGRES_USER", "user")?;
    let password = environment.require_variable("POSTGRES_PASSWORD", "password")?;
    let host = environment.variable_or("POSTGRES_HOST", "localhost");
    let port = environment.variable_or("POSTGRES_PORT", "2423");
    let db = environment.variable_or("POSTGRES_DB", "baihua");

    Ok(format!(
        "postgres://{}:{}@{}:{}/{}",
        user, password, host, port, db
    ))
}

pub async fn get_pool(
    environment: Environment,
    config: &DatabaseConfiguration,
) -> Result<Pool<Postgres>> {
    let url = build_connection_url(environment)?;

    sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .idle_timeout(std::time::Duration::from_secs(600))
        .max_lifetime(std::time::Duration::from_secs(3600))
        .connect(&url)
        .await
        .context("Failed to connect to database.")
}
