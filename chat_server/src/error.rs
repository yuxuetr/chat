use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorOutput {
  pub error: String,
}

#[derive(Error, Debug)]
pub enum AppError {
  #[error("email already exists: {0}")]
  EmailAlreadyExists(String),

  #[error("sql error: {0}")]
  SqlxError(#[from] sqlx::Error),

  #[error("password hash error: {0}")]
  PasswordHashError(#[from] argon2::password_hash::Error),

  #[error("jwt error: {0}")]
  JwtError(#[from] jwt_simple::Error),

  #[error("http header parse error: {0}")]
  HttpHeaderError(#[from] axum::http::header::InvalidHeaderValue),

  #[error("create chat error: {0}")]
  CreateChatError(String),

  #[error("update chat error: {0}")]
  UpdateChatError(String),

  #[error("delete chat error: {0}")]
  DeleteChatError(String),

  #[error("not found: {0}")]
  NotFound(String),

  #[error("io error: {0}")]
  IoError(#[from] std::io::Error),

  #[error("{0}")]
  ChatFileError(String),

  #[error("create message error: {0}")]
  CreateMessageError(String),
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response<axum::body::Body> {
    let status = match &self {
      Self::EmailAlreadyExists(_) => StatusCode::CONFLICT,
      Self::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      Self::PasswordHashError(_) => StatusCode::UNPROCESSABLE_ENTITY,
      Self::JwtError(_) => StatusCode::FORBIDDEN,
      Self::HttpHeaderError(_) => StatusCode::UNPROCESSABLE_ENTITY,
      Self::CreateChatError(_) => StatusCode::BAD_REQUEST,
      Self::NotFound(_) => StatusCode::NOT_FOUND,
      Self::UpdateChatError(_) => StatusCode::BAD_REQUEST,
      Self::DeleteChatError(_) => StatusCode::BAD_REQUEST,
      Self::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      Self::ChatFileError(_) => StatusCode::BAD_REQUEST,
      Self::CreateMessageError(_) => StatusCode::BAD_REQUEST,
    };

    (status, Json(ErrorOutput::new(self.to_string()))).into_response()
  }
}

impl ErrorOutput {
  pub fn new(error: impl Into<String>) -> Self {
    Self {
      error: error.into(),
    }
  }
}
