use crate::Directory;
use crate::ServerState;
use crate::config::AppConfig;
use crate::log;
use anyhow::{Context, Result, anyhow};
use dirs::home_dir;
use std::path::Path;
use tokio::fs;
use tracing::info;

pub async fn init() -> Result<(ServerState, tracing_appender::non_blocking::WorkerGuard)> {
    println!("Start initializing the server.");

    // Get the user home directory.
    let home = home_dir().context("The user home directory cannot be obtained.")?;
    let app_dir = home.join(".baihua");

    create_app_dirs(&app_dir).await?;

    // Load or create a configuration.
    let config = load_or_create_config(&app_dir).await?;
    config.validate()?;

    let directory = Directory {
        app: app_dir.clone(),
        log: app_dir.join("logs"),
    };

    let state = ServerState { directory, config };

    let guard = log::init_log(&state)
        .map_err(|e| anyhow!("Failed to initialize logging system: {}.", e))?;

    info!("Initialization completed.");

    Ok((state, guard))
}

async fn create_app_dirs(app_dir: &Path) -> Result<()> {
    let dirs = vec!["logs"];

    for dir_name in dirs {
        let dir_path = app_dir.join(dir_name);
        if !dir_path.exists() {
            fs::create_dir_all(&dir_path)
                .await
                .with_context(|| format!("Failed to create a directory:{:?}", dir_path))?;
            println!("Create a directory: {:?}", dir_path);
        }
    }

    Ok(())
}

async fn load_or_create_config(app_dir: &Path) -> Result<AppConfig> {
    let config_path = app_dir.join("config.toml");

    if config_path.exists() {
        // Load the configuration from an existing profile.
        let content = fs::read_to_string(&config_path)
            .await
            .context("Failed to read the profile.")?;

        let config: AppConfig = toml::from_str(&content).context("Failed to parse the profile.")?;

        println!("Load the configuration from an existing profile.");
        Ok(config)
    } else {
        // Create a profile.
        let config = AppConfig::default();
        let toml_content =
            toml::to_string_pretty(&config).context("Serialization configuration failed.")?;

        fs::write(&config_path, toml_content)
            .await
            .context("Write to the profile failed.")?;

        println!("Create a profile: {:?}", config_path);

        Ok(config)
    }
}
