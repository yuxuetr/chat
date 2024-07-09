use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::{env, fs::File};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
  pub server: ServerConfig,
  pub auth: AuthConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
  pub pk: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
  pub port: u16,
  pub db_url: String,
}

impl AppConfig {
  pub fn load() -> Result<Self> {
    let file_ops = (
      File::open("notif.yaml"),
      File::open("/etc/config/notif.yaml"),
      env::var("NOTIFY_CONFIG"),
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
