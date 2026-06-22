use anyhow::Result;
use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Development,
    Production,
}

impl Environment {
    pub fn from_env() -> Self {
        match env::var("BAIHUA_ENV").as_deref() {
            Ok("production") => Environment::Production,
            _ => Environment::Development,
        }
    }

    pub fn is_production(&self) -> bool {
        matches!(self, Environment::Production)
    }

    pub fn is_development(&self) -> bool {
        matches!(self, Environment::Development)
    }

    // In production: require the variable to exist, else hard failure.
    // In development: fall back to the provided default.
    pub fn require_var(&self, name: &str, dev_default: &str) -> Result<String> {
        match env::var(name) {
            Ok(val) => Ok(val),
            Err(_) => {
                if self.is_production() {
                    Err(anyhow::anyhow!(
                        "Required environment variable '{}' is not set.",
                        name
                    ))
                } else {
                    Ok(dev_default.to_string())
                }
            }
        }
    }

    // Read an environment variable with a fallback default.
    pub fn var_or(&self, name: &str, default: &str) -> String {
        env::var(name).unwrap_or_else(|_| default.to_string())
    }
}
