use crate::User;
use jwt_simple::prelude::*;
use std::ops::Deref;

const JWT_DURATION: u64 = 60 * 60 * 24 * 7;
const JWT_ISS: &str = "chat_server";
const JWT_AUD: &str = "chat_web";

pub struct EncodingKey(Ed25519KeyPair);
pub struct DecodingKey(Ed25519PublicKey);

#[allow(unused)]
impl EncodingKey {
  pub fn load(pem: &str) -> Result<Self, jwt_simple::Error> {
    Ok(Self(Ed25519KeyPair::from_pem(pem)?))
  }

  pub fn sign(&self, user: impl Into<User>) -> Result<String, jwt_simple::Error> {
    let user = user.into();
    let claims = Claims::with_custom_claims(user, Duration::from_secs(JWT_DURATION));
    let claims = claims.with_issuer(JWT_ISS).with_audience(JWT_AUD);
    self.0.sign(claims)
  }
}

#[allow(unused)]
impl DecodingKey {
  pub fn load(pem: &str) -> Result<Self, jwt_simple::Error> {
    Ok(Self(Ed25519PublicKey::from_pem(pem)?))
  }

  pub fn verify(&self, token: &str) -> Result<User, jwt_simple::Error> {
    let mut options = VerificationOptions {
      allowed_issuers: Some(HashSet::from_strings(&[JWT_ISS])),
      allowed_audiences: Some(HashSet::from_strings(&[JWT_AUD])),
      ..Default::default()
    };
    let claims = self.0.verify_token::<User>(token, Some(options))?;
    Ok(claims.custom)
  }
}

impl Deref for EncodingKey {
  type Target = Ed25519KeyPair;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl Deref for DecodingKey {
  type Target = Ed25519PublicKey;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::User;
  use anyhow::Result;

  #[tokio::test]
  async fn test_jwt() -> Result<()> {
    let encoding_pem = include_str!("../../fixtures/encoding.pem");
    let decoding_pem = include_str!("../../fixtures/decoding.pem");
    let ek = EncodingKey::load(encoding_pem)?;
    let dk = DecodingKey::load(decoding_pem)?;

    let user = User::new(1, "Hal", "halzzz@gmail.com");

    let token = ek.sign(user.clone())?;
    let user2 = dk.verify(&token)?;
    assert_eq!(user, user2);
    Ok(())
  }
}
