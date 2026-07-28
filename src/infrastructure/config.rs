use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfiguration {
    pub server: WebConfiguration,
    pub log: LogsConfiguration,
    pub database: DatabaseConfiguration,
    pub user: UserConfiguration,
}

// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfiguration {
    pub host: String,
    pub port: u16,
}

// Log configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogsConfiguration {
    #[serde(default = "default_log_level")]
    pub level: String, // default: info
}

// Database configuration
// Connection details are provided via environment variables to avoid
// hardcoding credentials in config files. See .env.example for the
// required POSTGRES_* variables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfiguration {
    #[serde(default = "default_max_connections")]
    pub max_connections: u32, // default: 20

    #[serde(default = "default_min_connections")]
    pub min_connections: u32, // default: 5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfiguration {
    #[serde(default = "default_username_length")]
    pub minimum_username_length: u32, // default: 3

    #[serde(default = "default_maximum_username_length")]
    pub maximum_username_length: u32, // default: 40

    #[serde(default = "default_jsonwebtoken_expiration_hours")]
    pub jsonwebtoken_expiration_hours: u32, // default: 24
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

fn default_username_length() -> u32 {
    3
}
fn default_maximum_username_length() -> u32 {
    40
}
fn default_jsonwebtoken_expiration_hours() -> u32 {
    24
}

impl Default for ServerConfiguration {
    fn default() -> Self {
        Self {
            server: WebConfiguration {
                host: "127.0.0.1".to_string(),
                port: 2424,
            },
            log: LogsConfiguration {
                level: default_log_level(),
            },
            database: DatabaseConfiguration {
                max_connections: default_max_connections(),
                min_connections: default_min_connections(),
            },
            user: UserConfiguration {
                minimum_username_length: default_username_length(),
                maximum_username_length: default_maximum_username_length(),
                jsonwebtoken_expiration_hours: default_jsonwebtoken_expiration_hours(),
            },
        }
    }
}

impl ServerConfiguration {
    pub fn validate(&self) -> Result<()> {
        // validate the profile information.
        if self.server.port == 0 {
            bail!("The server port cannot be 0.");
        }

        if self.database.max_connections < self.database.min_connections {
            bail!(
                "The maximum number of connections in the database cannot be less than the minimum number of connections."
            );
        }

        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.log.level.as_str()) {
            bail!("Invalid log level: {}", self.log.level);
        }

        if self.user.minimum_username_length > self.user.maximum_username_length {
            bail!(
                "The minimum username length cannot be greater than the maximum username length."
            );
        }

        if self.user.minimum_username_length == 0 || self.user.maximum_username_length == 0 {
            bail!("The minimum or maximum value of the username length cannot be 0.");
        }

        if self.user.jsonwebtoken_expiration_hours == 0 {
            bail!("The jsonwebtoken expiration hours cannot be 0.");
        }

        Ok(())
    }
}
