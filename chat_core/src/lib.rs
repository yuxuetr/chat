pub mod middlewares;
mod utils;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

pub use utils::*;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq)]
pub struct User {
  pub id: i64,
  pub ws_id: i64,
  pub fullname: String,
  pub email: String,
  #[sqlx(default)]
  #[serde(skip)]
  pub password_hash: Option<String>,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq)]
pub struct Workspace {
  pub id: i64,
  pub name: String,
  pub owner_id: i64,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq)]
pub struct ChatUser {
  pub id: i64,
  pub fullname: String,
  pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, sqlx::Type)]
#[sqlx(type_name = "chat_type", rename_all = "snake_case")]
pub enum ChatType {
  Single,
  Group,
  PrivateChannel,
  PublicChannel,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq)]
pub struct Chat {
  pub id: i64,
  pub ws_id: i64,
  pub name: Option<String>,
  pub r#type: ChatType,
  pub members: Vec<i64>,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq)]
pub struct Message {
  pub id: i64,
  pub chat_id: i64,
  pub sender_id: i64,
  pub content: String,
  pub files: Vec<String>,
  pub created_at: DateTime<Utc>,
}

impl User {
  pub fn new(id: i64, fullname: &str, email: &str) -> Self {
    Self {
      id,
      ws_id: 0,
      fullname: fullname.to_string(),
      email: email.to_string(),
      password_hash: None,
      created_at: chrono::Utc::now(),
    }
  }
}
