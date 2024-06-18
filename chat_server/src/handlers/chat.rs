use crate::User;
use axum::{response::IntoResponse, Extension};
use tracing::info;

pub(crate) async fn create_chat_handler(Extension(user): Extension<User>) -> impl IntoResponse {
  info!("user: {:?}", user);
  "create chat"
}
pub(crate) async fn list_chat_handler(Extension(user): Extension<User>) -> impl IntoResponse {
  info!("user: {:?}", user);
  "list chat"
}

pub(crate) async fn update_chat_handler() -> impl IntoResponse {
  "update chat"
}

pub(crate) async fn delete_chat_handler() -> impl IntoResponse {
  "delete chat"
}
