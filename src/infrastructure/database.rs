use crate::infrastructure::config::DatabaseConfigure;
use crate::infrastructure::environment::Environment;
use anyhow::{Context, Result};
use sqlx::{Pool, Postgres};

pub fn build_connection_url(env: Environment) -> Result<String> {
    let user = env.require_var("POSTGRES_USER", "user")?;
    let password = env.require_var("POSTGRES_PASSWORD", "password")?;
    let host = env.var_or("POSTGRES_HOST", "localhost");
    let port = env.var_or("POSTGRES_PORT", "2423");
    let db = env.var_or("POSTGRES_DB", "baihua");

    Ok(format!(
        "postgres://{}:{}@{}:{}/{}",
        user, password, host, port, db
    ))
}

pub async fn get_pool(env: Environment, config: &DatabaseConfigure) -> Result<Pool<Postgres>> {
    let url = build_connection_url(env)?;

    sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .idle_timeout(std::time::Duration::from_secs(600))
        .max_lifetime(std::time::Duration::from_secs(3600))
        .connect(&url)
        .await
        .context("Failed to connect to database.")
}
