use axum::response::IntoResponse;

pub(crate) async fn update_chat_handler() -> impl IntoResponse {
  "create chat"
}

pub(crate) async fn delete_chat_handler() -> impl IntoResponse {
  "delete chat"
}
