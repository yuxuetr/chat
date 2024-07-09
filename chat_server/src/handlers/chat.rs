use crate::{AppError, AppState, CreateChat, UpdateChat};
use axum::{
  extract::{Path, State},
  http::StatusCode,
  response::IntoResponse,
  Extension, Json,
};
use chat_core::User;
use tracing::info;

pub(crate) async fn create_chat_handler(
  Extension(user): Extension<User>,
  State(state): State<AppState>,
  Json(input): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
  info!("user: {:?}", user);
  let chat = state.create_chat(input, user.ws_id as _).await?;
  Ok((StatusCode::CREATED, Json(chat)))
}

pub(crate) async fn list_chat_handler(
  Extension(user): Extension<User>,
  State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
  let chat = state.fetch_chats(user.ws_id as _).await?;
  info!("user: {:?}", user);
  Ok((StatusCode::OK, Json(chat)))
}

pub(crate) async fn get_chat_handler(
  State(state): State<AppState>,
  Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
  let chat = state.get_chat_by_id(id).await?;
  match chat {
    Some(chat) => Ok(Json(chat)),
    None => Err(AppError::NotFound(format!("chat id: {id}"))),
  }
}

pub(crate) async fn update_chat_handler(
  Path(id): Path<u64>,
  State(state): State<AppState>,
  Json(input): Json<UpdateChat>,
) -> Result<impl IntoResponse, AppError> {
  let chat = state.update_chat(id, input).await?;
  Ok((StatusCode::ACCEPTED, Json(chat)))
}

pub(crate) async fn delete_chat_handler(
  State(state): State<AppState>,
  Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
  state.delete_chat(id).await?;
  Ok((StatusCode::OK, "Delete Successfully"))
}
