use chrono::Utc;
use jsonwebtoken::{Algorithm, encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
  pub uuid: Uuid,
  pub exp: usize,
}

pub fn create_jwt(uuid: Uuid) -> crate::Result<String> {
  let expiration = Utc::now()
    .checked_add_signed(chrono::Duration::hours(1))
    .expect("Valid timestamp")
    .timestamp();

  let claims = Claims {
    uuid,
    exp: expiration as usize,
  };

  let header = Header::new(Algorithm::HS512);
  Ok(encode(
    &header,
    &claims,
    &EncodingKey::from_secret(dotenv!("TOKEN_SECRET").as_bytes()),
  ).unwrap())
}