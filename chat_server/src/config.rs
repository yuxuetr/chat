use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::{env, fs::File, path::PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
  pub server: ServerConfig,
  pub auth: AuthConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
  pub sk: String,
  pub pk: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
  pub port: u16,
  pub db_url: String,
  pub base_dir: PathBuf,
}

impl AppConfig {
  pub fn load() -> Result<Self> {
    let file_ops = (
      File::open("app.yaml"),
      File::open("/etc/config/app.yaml"),
      env::var("CHAT_CONFIG"),
    );

    let ret = match file_ops {
      (Ok(reader), _, _) => serde_yaml::from_reader(reader),
      (_, Ok(reader), _) => serde_yaml::from_reader(reader),
      (_, _, Ok(path)) => serde_yaml::from_reader(File::open(path)?),
      _ => bail!("Config file not found"),
    };
    Ok(ret?)
  }
}
