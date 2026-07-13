use crate::Directory;
use crate::ServerState;
use crate::infrastructure::config::AppConfigure;
use crate::infrastructure::database::get_pool;
use crate::infrastructure::environment::Environment;
use crate::infrastructure::log;
use crate::websocket::connection::ConnectionManager;
use anyhow::{Context, Result, anyhow};
use bcrypt::{DEFAULT_COST, hash};
use chrono::Utc;
use dirs::home_dir;
use jsonwebtoken::crypto::{CryptoProvider, rust_crypto::DEFAULT_PROVIDER};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tracing::info;
use uuid::Uuid;

pub async fn initialize(
    env: Environment,
) -> Result<(ServerState, tracing_appender::non_blocking::WorkerGuard)> {
    if env.is_development() {
        println!("Start initializing the server.");
    }

    // Determine the application directory.
    let home = home_dir().context("The user home directory cannot be obtained.")?;
    let app_directory = std::env::var("BAIHUA_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| home.join(".baihua"));

    create_app_directories(&app_directory, env).await?;

    // Load or create a configuration.
    let configuration = load_or_create_profiles(&app_directory, env).await?;
    configuration.validate()?;

    let directory = Directory {
        app: app_directory.clone(),
        log: app_directory.join("logs"),
    };

    let guard = log::init_log(&directory, &configuration, env)
        .map_err(|error| anyhow!("Failed to initialize logging system: {}.", error))?;

    let pool = get_pool(env, &configuration.database).await?;

    let migrations_path = if env.is_production() {
        let exe = std::env::current_exe()
            .context("Failed to determine binary location for migrations.")?;
        let exe_dir = exe
            .parent()
            .context("Binary path has no parent directory.")?
            .to_path_buf();
        exe_dir.join("migrations")
    } else if env.is_development() {
        std::path::PathBuf::from("migrations")
    } else {
        unreachable!()
    };

    let migrator = sqlx::migrate::Migrator::new(migrations_path)
        .await
        .context("Failed to load migrations.")?;
    migrator
        .run(&pool)
        .await
        .context("Failed to run migrations.")?;
    info!("Database initialized.");

    // Seed a default admin user in development mode.
    if env.is_development() {
        seed_dev_user(&pool).await?;
    }

    CryptoProvider::install_default(&DEFAULT_PROVIDER)
        .expect("Failed to install the default crypto provider.");

    let jwt_secret = env
        .require_var("JWT_SECRET", "your-default-jwt-secret-key")
        .context("Failed to load JWT secret.")?;

    info!("Initialization completed.");

    let state = ServerState {
        configure: configuration,
        pool,
        jwt_secret,
        environment: env,
        connection_manager: Arc::new(ConnectionManager::new()),
    };

    Ok((state, guard))
}

async fn create_app_directories(app_directory: &Path, env: Environment) -> Result<()> {
    let directories = vec!["logs"];

    for directory_name in directories {
        let directory_path = app_directory.join(directory_name);
        if !directory_path.exists() {
            fs::create_dir_all(&directory_path)
                .await
                .with_context(|| format!("Failed to create a directory:{:?}", directory_path))?;
            if env.is_development() {
                println!("Create a directory: {:?}", directory_path);
            }
        }
    }

    Ok(())
}

async fn load_or_create_profiles(app_directory: &Path, env: Environment) -> Result<AppConfigure> {
    let configure_path = app_directory.join("config.toml");

    if configure_path.exists() {
        let content = fs::read_to_string(&configure_path)
            .await
            .context("Failed to read the profile.")?;

        let config: AppConfigure =
            toml::from_str(&content).context("Failed to parse the profile.")?;

        if env.is_development() {
            println!("Load the configuration from an existing profile.");
        }
        Ok(config)
    } else {
        let configure = AppConfigure::default();
        let toml_content =
            toml::to_string_pretty(&configure).context("Serialization configuration failed.")?;

        fs::write(&configure_path, toml_content)
            .await
            .context("Write to the profile failed.")?;

        if env.is_development() {
            println!("Create a profile: {:?}", configure_path);
        }

        Ok(configure)
    }
}

async fn seed_dev_user(pool: &sqlx::Pool<sqlx::Postgres>) -> Result<()> {
    // Check if the dev seed user already exists.
    let existing = sqlx::query("SELECT id FROM users WHERE username = $1")
        .bind("admin")
        .fetch_optional(pool)
        .await
        .context("Failed to check for existing seed user.")?;

    if existing.is_some() {
        return Ok(());
    }

    let password = std::env::var("SEED_PASSWORD").unwrap_or_else(|_| "admin".to_string());
    let password_hash =
        hash(password, DEFAULT_COST).context("Failed to hash seed user password.")?;

    let id = Uuid::now_v7();
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO users (id, username, email, password, nickname, created_at, is_active) VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(id)
    .bind("admin")
    .bind("admin@localhost")
    .bind(&password_hash)
    .bind(Option::<String>::None)
    .bind(now)
    .bind(true)
    .execute(pool)
    .await
    .context("Failed to insert seed user.")?;

    info!("Seed user created: admin (password from SEED_PASSWORD env var, defaults to 'admin').");
    Ok(())
}
