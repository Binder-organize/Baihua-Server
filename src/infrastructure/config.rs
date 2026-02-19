use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfigure {
    pub server: ServerConfigure,
    pub log: LogConfigure,
    pub database: DatabaseConfigure,
}

// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfigure {
    pub host: String,
    pub port: u16,
}

// Log configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfigure {
    #[serde(default = "default_log_level")]
    pub level: String, // default: info
}

// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfigure {
    pub url: String,

    #[serde(default = "default_max_connections")]
    pub max_connections: u32, // default: 20

    #[serde(default = "default_min_connections")]
    pub min_connections: u32, // default: 5
}

// Default value function
fn default_log_level() -> String {
    "info".to_string()
}
fn default_max_connections() -> u32 {
    20
}
fn default_min_connections() -> u32 {
    5
}

impl Default for AppConfigure {
    fn default() -> Self {
        Self {
            server: ServerConfigure {
                host: "127.0.0.1".to_string(),
                port: 2424,
            },
            log: LogConfigure {
                level: default_log_level(),
            },
            database: DatabaseConfigure {
                url: "postgres://localhost/baihua".to_string(),
                max_connections: default_max_connections(),
                min_connections: default_min_connections(),
            },
        }
    }
}

impl AppConfigure {
    pub fn validate(&self) -> anyhow::Result<()> {
        // validate the profile information.
        if self.server.port == 0 {
            anyhow::bail!("The server port cannot be 0.");
        }

        if self.database.max_connections < self.database.min_connections {
            anyhow::bail!(
                "The maximum number of connections in the database cannot be less than the minimum number of connections."
            );
        }

        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.log.level.as_str()) {
            anyhow::bail!("Invalid log level: {}", self.log.level);
        }

        Ok(())
    }
}
