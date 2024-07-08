mod config;
mod error;
mod handlers;
mod middlewares;
mod models;
mod utils;

use anyhow::Context;
use axum::{
  middleware::from_fn_with_state,
  routing::{delete, get, patch, post},
  Router,
};
pub use config::AppConfig;
use error::*;
use handlers::*;
use middlewares::{set_layer, verify_chat, verify_token};
use models::*;
use sqlx::PgPool;
use std::{fmt, ops::Deref, sync::Arc};
use tokio::fs;
use utils::{DecodingKey, EncodingKey};

#[derive(Debug, Clone)]
pub(crate) struct AppState {
  inner: Arc<AppStateInner>,
}

#[allow(unused)]
pub(crate) struct AppStateInner {
  pub(crate) config: AppConfig,
  pub(crate) dk: DecodingKey,
  pub(crate) ek: EncodingKey,
  pub(crate) pool: PgPool,
}

pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
  let state = AppState::try_new(config).await?;

  let chat = Router::new()
    .route("/:id", get(get_chat_handler))
    .route("/:id", patch(update_chat_handler))
    .route("/:id", delete(delete_chat_handler))
    .route("/:id", post(send_message_handler))
    .route("/:id/messages", get(list_message_handler))
    .layer(from_fn_with_state(state.clone(), verify_chat))
    .route("/", get(list_chat_handler).post(create_chat_handler));

  let api = Router::new()
    .nest("/chats", chat)
    .route("/upload", post(upload_handler))
    .route("/files/:ws_id/*path", get(file_handler))
    .layer(from_fn_with_state(state.clone(), verify_token))
    .route("/signin", post(signin_handler))
    .route("/signup", post(signup_handler));

  let app = Router::new()
    .route("/", get(index_handler))
    .nest("/api", api)
    .with_state(state);

  Ok(set_layer(app))
}

// state.config => state.inner.config
impl Deref for AppState {
  type Target = AppStateInner;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl AppState {
  pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
    fs::create_dir_all(&config.server.base_dir)
      .await
      .context("create base_dir failed")?;
    let dk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
    let ek = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
    let pool = PgPool::connect(&config.server.db_url)
      .await
      .context("connect to db failed")?;
    Ok(Self {
      inner: Arc::new(AppStateInner {
        config,
        ek,
        dk,
        pool,
      }),
    })
  }
}

impl fmt::Debug for AppStateInner {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("AppStateInner")
      .field("config", &self.config)
      .finish()
  }
}

#[cfg(test)]
mod test_util {
  use super::*;
  use sqlx::{Executor, PgPool};
  use sqlx_db_tester::TestPg;

  impl AppState {
    pub async fn new_for_test() -> Result<(TestPg, Self), AppError> {
      let config = AppConfig::load()?;
      let dk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
      let ek = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
      let post = config.server.db_url.rfind('/').expect("invalid db_url");
      let server_url = &config.server.db_url[..post];
      let (tdb, pool) = get_test_pool(Some(server_url)).await;
      let state = Self {
        inner: Arc::new(AppStateInner {
          config,
          dk,
          ek,
          pool,
        }),
      };
      Ok((tdb, state))
    }
  }

  pub async fn get_test_pool(url: Option<&str>) -> (TestPg, PgPool) {
    let url = match url {
      Some(url) => url.to_string(),
      None => "postgres://postgres:123456@localhost:5432".to_string(),
    };
    let tdb = TestPg::new(url, std::path::Path::new("../migrations"));
    let pool = tdb.get_pool().await;

    let sql = include_str!("../fixtures/test.sql").split(';');
    let mut ts = pool.begin().await.expect("begin transaction failed");
    for s in sql {
      if s.trim().is_empty() {
        continue;
      }
      ts.execute(s).await.expect("execute sql failed");
    }
    ts.commit().await.expect("commit transaction failed");

    (tdb, pool)
  }
}
