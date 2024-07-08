use crate::{
  error::{AppError, ErrorOutput},
  models::{CreateUser, SigninUser},
  AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthOutput {
  token: String,
}

pub(crate) async fn signup_handler(
  State(state): State<AppState>,
  Json(input): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
  let user = state.create_user(&input).await?;
  let token = state.ek.sign(user)?;
  let body = Json(AuthOutput { token });
  Ok((StatusCode::CREATED, body))
}

pub(crate) async fn signin_handler(
  State(state): State<AppState>,
  Json(input): Json<SigninUser>,
) -> Result<impl IntoResponse, AppError> {
  let user = state.verify_user(&input).await?;

  match user {
    Some(user) => {
      let token = state.ek.sign(user)?;
      Ok((StatusCode::OK, Json(AuthOutput { token })).into_response())
    }
    None => {
      let body = Json(ErrorOutput::new("Invalid email or password"));
      Ok((StatusCode::FORBIDDEN, body).into_response())
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use anyhow::Result;
  use http_body_util::BodyExt;

  #[tokio::test]
  async fn signup_should_work() -> Result<()> {
    let (_tdb, state) = AppState::new_for_test().await?;
    let input = CreateUser::new("acme", "Hal", "halzzz@gmail.com", "halzzz");
    let ret = signup_handler(State(state), Json(input))
      .await?
      .into_response();
    assert_eq!(ret.status(), StatusCode::CREATED);

    let body = ret.into_body().collect().await?.to_bytes();
    let ret: AuthOutput = serde_json::from_slice(&body)?;
    assert_ne!(ret.token, "");
    Ok(())
  }

  #[tokio::test]
  async fn signup_duplicate_user_should_409() -> Result<()> {
    let (_tdb, state) = AppState::new_for_test().await?;
    let input = CreateUser::new("acme", "Hal3", "halzzz3@gmail.com", "halzzz3");
    signup_handler(State(state.clone()), Json(input.clone())).await?;
    let ret = signup_handler(State(state), Json(input.clone()))
      .await
      .into_response();
    assert_eq!(ret.status(), StatusCode::CONFLICT);

    let body = ret.into_body().collect().await?.to_bytes();
    let ret: ErrorOutput = serde_json::from_slice(&body)?;
    assert_eq!(ret.error, "email already exists: halzzz3@gmail.com");
    Ok(())
  }

  #[tokio::test]
  async fn signin_should_work() -> Result<()> {
    let (_tdb, state) = AppState::new_for_test().await?;
    let email = "halzzz@acme.com";
    let password = "123456";

    // create first
    let create_info = CreateUser::new("acme", "Test User", email, password);
    state.create_user(&create_info).await?;

    // signin then
    let input = SigninUser::new(email, password);
    let ret = signin_handler(State(state), Json(input))
      .await?
      .into_response();
    assert_eq!(ret.status(), StatusCode::OK);
    let body = ret.into_body().collect().await?.to_bytes();
    let ret: AuthOutput = serde_json::from_slice(&body)?;
    assert_ne!(ret.token, "");

    Ok(())
  }

  #[tokio::test]
  async fn signin_with_non_exist_user_should_403() -> Result<()> {
    let (_tdb, state) = AppState::new_for_test().await?;
    let email = "xuetr@gmail.com";
    let password = "xuetr";
    let input = SigninUser::new(email, password);
    let ret = signin_handler(State(state), Json(input))
      .await
      .into_response();
    assert_eq!(ret.status(), StatusCode::FORBIDDEN);
    let body = ret.into_body().collect().await?.to_bytes();
    let ret: ErrorOutput = serde_json::from_slice(&body)?;
    assert_eq!(ret.error, "Invalid email or password");
    Ok(())
  }
}
