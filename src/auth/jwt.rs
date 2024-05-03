use chrono::Utc;
use jsonwebtoken::{Algorithm, encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const EXPIRATION: chrono::Duration = chrono::Duration::hours(1);

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
  pub uuid: Uuid,
  pub exp: usize,
}

// Creates token
pub fn create_jwt(uuid: Uuid, key: &EncodingKey) -> Result<String, jsonwebtoken::errors::Error> {
  // Expires after 1 hour
  let expiration = Utc::now()
    .checked_add_signed(EXPIRATION)
    .expect("Date out of range")
    .timestamp();

  let claims = Claims {
    uuid,
    exp: expiration as usize,
  };

  let header = Header::new(Algorithm::HS512);
  let token = encode(&header, &claims, key)?;

  Ok(token)
}
