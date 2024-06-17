use crate::{AppError, User};
use argon2::{
  password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
  Argon2,
};
use sqlx::PgPool;
use std::mem;

use super::{CreateUser, SigninUser};

#[allow(dead_code)]
impl User {
  pub async fn find_by_email(email: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
    let user = sqlx::query_as("SELECT id, fullname, email, created_at FROM users WHERE email = $1")
      .bind(email)
      .fetch_optional(pool)
      .await?;
    Ok(user)
  }

  pub async fn create(input: &CreateUser, pool: &PgPool) -> Result<Self, AppError> {
    let password_hash = hash_password(&input.password)?;

    let user = Self::find_by_email(&input.email, pool).await?;
    if user.is_some() {
      return Err(AppError::EmailAlreadyExists(input.email.clone()));
    }
    let user = sqlx::query_as(
      r#"
      INSERT INTO users (email, fullname, password_hash)
      VALUES ($1, $2, $3)
      RETURNING id, fullname, email, created_at
      "#,
    )
    .bind(&input.email)
    .bind(&input.fullname)
    .bind(password_hash)
    .fetch_one(pool)
    .await?;
    Ok(user)
  }

  pub async fn verify(input: &SigninUser, pool: &PgPool) -> Result<Option<Self>, AppError> {
    let user: Option<User> = sqlx::query_as(
      r#"SELECT id, fullname, email, password_hash, created_at from users WHERE
    email = $1"#,
    )
    .bind(&input.email)
    .fetch_optional(pool)
    .await?;
    match user {
      Some(mut user) => {
        let password_hash = mem::take(&mut user.password_hash);
        let is_valid = verify_password(&input.password, &password_hash.unwrap_or_default())?;
        if is_valid {
          Ok(Some(user))
        } else {
          Ok(None)
        }
      }
      None => Ok(None),
    }
  }
}

#[allow(unused)]
fn hash_password(password: &str) -> Result<String, AppError> {
  let salt = SaltString::generate(&mut OsRng);
  let argon2 = Argon2::default();
  let password_hash = argon2
    .hash_password(password.as_bytes(), &salt)?
    .to_string();
  Ok(password_hash)
}

#[allow(unused)]
fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
  let argon2 = Argon2::default();
  let parsed_hash = PasswordHash::new(password_hash)?;
  let is_valid = argon2
    .verify_password(password.as_bytes(), &parsed_hash)
    .is_ok();
  Ok(is_valid)
}

#[cfg(test)]
mod tests {
  use super::*;
  use anyhow::Result;
  use sqlx_db_tester::TestPg;
  use std::path::Path;

  #[test]
  fn hash_password_and_verify_should_work() -> Result<()> {
    let password = "halzzz";
    let password_hash = hash_password(password)?;
    assert_eq!(password_hash.len(), 97);
    assert!(verify_password(password, &password_hash)?);
    Ok(())
  }

  #[tokio::test]
  async fn create_and_verify_should_work() -> Result<()> {
    let tdb = TestPg::new(
      "postgres://postgres:postgres@localhost:5432".to_string(),
      Path::new("../migrations"),
    );
    let pool = tdb.get_pool().await;
    let email = "hal@gmail.com";
    let name = "Halzzz";
    let password = "halzzz";

    let input = CreateUser::new(name, email, password);
    let user = User::create(&input, &pool).await?;
    assert_eq!(user.email, email);
    assert_eq!(user.fullname, name);
    assert!(user.id > 0);

    let user = User::find_by_email(&input.email, &pool).await?;
    assert!(user.is_some());
    let user = user.unwrap();
    assert_eq!(user.email, input.email);
    assert_eq!(user.fullname, input.fullname);

    let input = SigninUser::new(&input.email, &input.password);
    let user = User::verify(&input, &pool).await?;
    assert!(user.is_some());

    Ok(())
  }
}
