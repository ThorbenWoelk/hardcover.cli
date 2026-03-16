use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub api_key: Option<String>,
}

fn config_dir() -> Result<PathBuf> {
    let base = dirs::config_dir().context("Could not determine config directory")?;
    Ok(base.join("hc"))
}

fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

pub fn load() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }
    let contents =
        fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
    let config: Config =
        toml::from_str(&contents).with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(config)
}

pub fn save(config: &Config) -> Result<()> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir).with_context(|| format!("Failed to create {}", dir.display()))?;
    let path = config_path()?;
    let contents = toml::to_string_pretty(config).context("Failed to serialize config")?;
    fs::write(&path, contents).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

/// Resolve API key with precedence: env var > config file.
pub fn resolve_api_key() -> Result<String> {
    if let Ok(key) = std::env::var("HARDCOVER_API_KEY") {
        return Ok(key);
    }

    let config = load()?;
    if let Some(key) = config.api_key {
        return Ok(key);
    }

    anyhow::bail!("No API key found. Run `hc login` or set HARDCOVER_API_KEY environment variable.")
}

/// Interactive login: prompt for token, validate, save.
pub fn login_interactive() -> Result<()> {
    print!("Paste your Hardcover API token: ");
    io::stdout().flush()?;

    let mut token = String::new();
    io::stdin().read_line(&mut token)?;
    let token = token.trim().to_string();

    if token.is_empty() {
        anyhow::bail!("Token cannot be empty");
    }

    let config = Config {
        api_key: Some(token),
    };
    save(&config)?;

    let path = config_path()?;
    println!("Saved to {}", path.display());
    Ok(())
}

/// Remove stored credentials.
pub fn logout() -> Result<()> {
    let path = config_path()?;
    if path.exists() {
        fs::remove_file(&path).with_context(|| format!("Failed to remove {}", path.display()))?;
        println!("Logged out. Removed {}", path.display());
    } else {
        println!("No config file found. Already logged out.");
    }
    Ok(())
}
