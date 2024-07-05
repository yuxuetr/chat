use super::{Chat, ChatType, ChatUser};
use crate::AppError;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreateChat {
  pub name: Option<String>,
  pub members: Vec<i64>,
  pub public: bool,
}

#[allow(unused)]
impl Chat {
  pub async fn create(input: CreateChat, ws_id: u64, pool: &PgPool) -> Result<Self, AppError> {
    let len = input.members.len();
    if len < 2 {
      return Err(AppError::CreateChatError(
        "Chat must have at least 2 members".to_string(),
      ));
    }

    if len > 8 && input.name.is_none() {
      return Err(AppError::CreateChatError(
        "Group chat with more than 8 members must have a name".to_string(),
      ));
    }

    let users = ChatUser::fetch_by_id(&input.members, pool).await?;
    if users.len() != len {
      return Err(AppError::CreateChatError(
        "Some members do not exist".to_string(),
      ));
    }

    let chat_type = match (&input.name, len) {
      (None, 2) => ChatType::Single,
      (None, _) => ChatType::Group,
      (Some(_), _) => {
        if input.public {
          ChatType::PublicChannel
        } else {
          ChatType::PrivateChannel
        }
      }
    };

    let chat = sqlx::query_as(
      r#"
      INSERT INTO chats (ws_id, name, type, members)
      VALUES ($1, $2, $3, $4)
      RETURNING id, ws_id, name, type, members, created_at
      "#,
    )
    .bind(ws_id as i64)
    .bind(input.name)
    .bind(chat_type)
    .bind(input.members)
    .fetch_one(pool)
    .await?;

    Ok(chat)
  }

  pub async fn fetch_all(ws_id: u64, pool: &PgPool) -> Result<Vec<Self>, AppError> {
    let chats = sqlx::query_as(
      r#"
      SELECT id, ws_id, name, type, members, created_at
      FROM chats
      WHERE ws_id = $1
      "#,
    )
    .bind(ws_id as i64)
    .fetch_all(pool)
    .await?;

    Ok(chats)
  }

  pub async fn get_by_id(id: u64, pool: &PgPool) -> Result<Option<Self>, AppError> {
    let chat = sqlx::query_as(
      r#"
      SELECT id, ws_id, name, type, members, created_at
      FROM chats
      WHERE id = $1
      "#,
    )
    .bind(id as i64)
    .fetch_optional(pool)
    .await?;

    Ok(chat)
  }
}

#[cfg(test)]
impl CreateChat {
  pub fn new(name: &str, members: &[i64], public: bool) -> Self {
    let name = if name.is_empty() {
      None
    } else {
      Some(name.to_string())
    };
    Self {
      name,
      members: members.to_vec(),
      public,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_util::get_test_pool;

  #[tokio::test]
  async fn create_single_chat_should_work() {
    let (_tdb, pool) = get_test_pool(None).await;
    let input = CreateChat::new("", &[1, 2], false);

    let chat = Chat::create(input, 1, &pool)
      .await
      .expect("create chat failed");
    assert_eq!(chat.ws_id, 1);
    assert_eq!(chat.members.len(), 2);
    assert_eq!(chat.r#type, ChatType::Single);
  }
}
