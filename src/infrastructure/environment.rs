use anyhow::Result;
use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Development,
    Production,
}

impl Environment {
    pub fn from_environment() -> Self {
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
    pub fn require_variable(&self, name: &str, development_default: &str) -> Result<String> {
        match env::var(name) {
            Ok(value) => Ok(value),
            Err(_) => {
                if self.is_production() {
                    Err(anyhow::anyhow!(
                        "Required environment variable '{}' is not set.",
                        name
                    ))
                } else if self.is_development() {
                    Ok(development_default.to_string())
                } else {
                    unreachable!()
                }
            }
        }
    }

    // Read an environment variable with a fallback default.
    pub fn variable_or(&self, name: &str, default: &str) -> String {
        env::var(name).unwrap_or_else(|_| default.to_string())
    }
}
