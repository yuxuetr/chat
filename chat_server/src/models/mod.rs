mod user;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq)]
pub struct User {
  pub id: i64,
  pub fullname: String,
  pub email: String,
  #[sqlx(default)]
  #[serde(skip)]
  pub password_hash: Option<String>,
  pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateUser {
  pub fullname: String,
  pub email: String,
  pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SigninUser {
  pub email: String,
  pub password: String,
}

#[cfg(test)]
impl User {
  pub fn new(id: i64, fullname: &str, email: &str) -> Self {
    Self {
      id,
      fullname: fullname.to_string(),
      email: email.to_string(),
      password_hash: None,
      created_at: Utc::now(),
    }
  }
}

#[cfg(test)]
impl CreateUser {
  pub fn new(fullname: &str, email: &str, password: &str) -> Self {
    Self {
      fullname: fullname.to_string(),
      email: email.to_string(),
      password: password.to_string(),
    }
  }
}

#[cfg(test)]
impl SigninUser {
  pub fn new(email: &str, password: &str) -> Self {
    Self {
      email: email.to_string(),
      password: password.to_string(),
    }
  }
}
