use anyhow::{Context, Result};
use dotenvy::dotenv;
use sqlx::{Pool, Postgres, query};

pub async fn get_pool() -> Result<Pool<Postgres>> {
    dotenv().context("Failed to load .env file")?;

    // todo Centralize reads of environment variables into a separate module and do it when the server is initialized.
    let user = dotenvy::var("POSTGRES_USER").unwrap_or_else(|_| "user".to_string());
    let password = dotenvy::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "password".to_string());
    let host = dotenvy::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = dotenvy::var("POSTGRES_PORT").unwrap_or_else(|_| "2423".to_string());
    let db = dotenvy::var("POSTGRES_DB").unwrap_or_else(|_| "baihua".to_string());

    let url = format!("postgres://{}:{}@{}:{}/{}", user, password, host, port, db);
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .context("Failed to connect to database.")
}

pub async fn initialize_database(pool: &Pool<Postgres>) -> Result<()> {
    query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            email TEXT NOT NULL UNIQUE,
            password TEXT NOT NULL,
            nickname TEXT,
            phone_number TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            is_active BOOLEAN NOT NULL DEFAULT true
        )
        "#,
    )
    .execute(pool)
    .await
    .context("Failed to create users table.")?;

    Ok(())
}
