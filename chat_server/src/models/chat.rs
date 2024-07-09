use crate::{AppError, AppState};
use chat_core::{Chat, ChatType};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreateChat {
  pub name: Option<String>,
  pub members: Vec<i64>,
  pub public: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateChat {
  pub name: Option<String>,
  pub members: Option<Vec<i64>>,
  pub public: Option<bool>,
}

#[allow(unused)]
impl AppState {
  pub async fn create_chat(&self, input: CreateChat, ws_id: u64) -> Result<Chat, AppError> {
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

    let users = self.fetch_chat_user_by_ids(&input.members).await?;
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
    .fetch_one(&self.pool)
    .await?;

    Ok(chat)
  }

  pub async fn fetch_chats(&self, ws_id: u64) -> Result<Vec<Chat>, AppError> {
    let chats = sqlx::query_as(
      r#"
      SELECT id, ws_id, name, type, members, created_at
      FROM chats
      WHERE ws_id = $1
      "#,
    )
    .bind(ws_id as i64)
    .fetch_all(&self.pool)
    .await?;

    Ok(chats)
  }

  pub async fn get_chat_by_id(&self, id: u64) -> Result<Option<Chat>, AppError> {
    let chat = sqlx::query_as(
      r#"
      SELECT id, ws_id, name, type, members, created_at
      FROM chats
      WHERE id = $1
      "#,
    )
    .bind(id as i64)
    .fetch_optional(&self.pool)
    .await?;

    Ok(chat)
  }

  pub async fn is_chat_member(&self, chat_id: u64, user_id: u64) -> Result<bool, AppError> {
    let is_member = sqlx::query(
      r#"
      SELECT 1
      FROM chats
      WHERE id = $1 AND $2 = ANY(members)
      "#,
    )
    .bind(chat_id as i64)
    .bind(user_id as i64)
    .fetch_optional(&self.pool)
    .await?;

    Ok(is_member.is_some())
  }

  // update chat info
  pub async fn update_chat(&self, id: u64, input: UpdateChat) -> Result<Chat, AppError> {
    let mut chat = self.get_chat_by_id(id).await?;
    info!("{:?}", chat);
    let mut chat = match chat {
      Some(chat) => chat,
      None => return Err(AppError::NotFound("Chat: {id} not found".to_string())),
    };

    if let Some(name) = input.name {
      chat.name = Some(name);
    }

    if let Some(members) = input.members {
      let users = self.fetch_chat_user_by_ids(&members).await?;
      if users.len() != members.len() {
        return Err(AppError::UpdateChatError(
          "Some members do not exist".to_string(),
        ));
      }
      chat.members = members;
    }

    if let Some(public) = input.public {
      chat.r#type = if public {
        ChatType::PublicChannel
      } else {
        ChatType::PrivateChannel
      };
    }

    let chat = sqlx::query_as(
      r#"
      UPDATE chats
      SET name = $1, members = $2, type = $3
      WHERE id = $4
      RETURNING id, ws_id, name, type, members, created_at
      "#,
    )
    .bind(chat.name)
    .bind(chat.members)
    .bind(chat.r#type)
    .bind(id as i64)
    .fetch_one(&self.pool)
    .await?;

    Ok(chat)
  }

  pub async fn delete_chat(&self, id: u64) -> Result<(), AppError> {
    let chat = self.get_chat_by_id(id).await?;
    let chat = match chat {
      Some(chat) => chat,
      None => return Err(AppError::NotFound(format!("Chat: {} not found", id))),
    };

    sqlx::query(
      r#"
      DELETE FROM chats
      WHERE id = $1
      "#,
    )
    .bind(id as i64)
    .execute(&self.pool)
    .await?;

    Ok(())
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
  use anyhow::Result;

  #[tokio::test]
  async fn create_single_chat_should_work() -> Result<()> {
    let (_tdb, state) = AppState::new_for_test().await?;
    let input = CreateChat::new("", &[1, 2], false);

    let chat = state
      .create_chat(input, 1)
      .await
      .expect("create chat failed");
    assert_eq!(chat.ws_id, 1);
    assert_eq!(chat.members.len(), 2);
    assert_eq!(chat.r#type, ChatType::Single);
    Ok(())
  }

  #[tokio::test]
  async fn create_public_named_chat_should_work() -> Result<()> {
    let (_tdb, state) = AppState::new_for_test().await?;
    let input = CreateChat::new("general", &[1, 2, 3], true);
    let chat = state
      .create_chat(input, 1)
      .await
      .expect("create chat failed");
    assert_eq!(chat.ws_id, 1);
    assert_eq!(chat.members.len(), 3);
    assert_eq!(chat.r#type, ChatType::PublicChannel);
    Ok(())
  }

  #[tokio::test]
  async fn chat_get_by_id_should_work() -> Result<()> {
    let (_tdb, state) = AppState::new_for_test().await?;
    let chat = state
      .get_chat_by_id(1)
      .await
      .expect("get chat by id failed")
      .unwrap();
    assert_eq!(chat.id, 1);
    assert_eq!(chat.name.unwrap(), "general");
    assert_eq!(chat.ws_id, 1);
    assert_eq!(chat.members.len(), 5);
    Ok(())
  }

  #[tokio::test]
  async fn chat_fetch_all_should_work() -> Result<()> {
    let (_tdb, state) = AppState::new_for_test().await?;
    let chats = state.fetch_chats(1).await.expect("fetch all chats failed");
    assert_eq!(chats.len(), 4);
    Ok(())
  }

  #[tokio::test]
  async fn chat_is_member_should_work() -> Result<()> {
    let (_tdb, state) = AppState::new_for_test().await?;
    let is_member = state.is_chat_member(1, 1).await.expect("is member failed");
    assert!(is_member);

    let is_member = state.is_chat_member(1, 6).await.expect("is member failed");
    assert!(!is_member);

    let is_member = state.is_chat_member(10, 1).await.expect("is member failed");
    assert!(!is_member);

    let is_member = state.is_chat_member(2, 4).await.expect("is member failed");
    assert!(!is_member);

    Ok(())
  }
}
