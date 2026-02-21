use crate::Directory;
use crate::ServerState;
use crate::infrastructure::config::AppConfigure;
use crate::infrastructure::database::{get_pool, initialize_database};
use crate::infrastructure::log;
use anyhow::{Context, Result, anyhow};
use dirs::home_dir;
use std::path::Path;
use tokio::fs;
use tracing::info;

pub async fn initialize() -> Result<(ServerState, tracing_appender::non_blocking::WorkerGuard)> {
    println!("Start initializing the server.");

    // Get the user home directory.
    let home = home_dir().context("The user home directory cannot be obtained.")?;
    let app_directory = home.join(".baihua");

    create_app_directories(&app_directory).await?;

    // Load or create a configuration.
    let configuration = load_or_create_profiles(&app_directory).await?;
    configuration.validate()?;

    let directory = Directory {
        app: app_directory.clone(),
        log: app_directory.join("logs"),
    };

    let guard = log::init_log(&directory, &configuration)
        .map_err(|e| anyhow!("Failed to initialize logging system: {}.", e))?;

    let pool = get_pool().await?;
    initialize_database(&pool).await?;
    info!("Database initialized.");

    info!("Initialization completed.");

    let state = ServerState {
        configure: configuration,
        pool,
    };

    Ok((state, guard))
}

async fn create_app_directories(app_directory: &Path) -> Result<()> {
    let directories = vec!["logs"];

    for directory_name in directories {
        let directory_path = app_directory.join(directory_name);
        if !directory_path.exists() {
            fs::create_dir_all(&directory_path)
                .await
                .with_context(|| format!("Failed to create a directory:{:?}", directory_path))?;
            println!("Create a directory: {:?}", directory_path);
        }
    }

    Ok(())
}

async fn load_or_create_profiles(app_directory: &Path) -> Result<AppConfigure> {
    let configure_path = app_directory.join("config.toml");

    if configure_path.exists() {
        // Load the configuration from an existing profile.
        let content = fs::read_to_string(&configure_path)
            .await
            .context("Failed to read the profile.")?;

        let config: AppConfigure =
            toml::from_str(&content).context("Failed to parse the profile.")?;

        println!("Load the configuration from an existing profile.");
        Ok(config)
    } else {
        // Create a profile.
        let configure = AppConfigure::default();
        let toml_content =
            toml::to_string_pretty(&configure).context("Serialization configuration failed.")?;

        fs::write(&configure_path, toml_content)
            .await
            .context("Write to the profile failed.")?;

        println!("Create a profile: {:?}", configure_path);

        Ok(configure)
    }
}
