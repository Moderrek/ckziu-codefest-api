use chrono::Utc;
use jsonwebtoken::{Algorithm, encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use warp::Reply;
use warp::reply::json;
use crate::{WebResult};
use crate::models::{LoginRequest, LoginResponse};

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
  uid: String,
  role: String,
  exp: usize,
}

pub fn create_jwt(uid: &str, role: &str) -> crate::Result<String> {
  let expiration = Utc::now()
    .checked_add_signed(chrono::Duration::seconds(60))
    .expect("Valid timestamp")
    .timestamp();

  let claims = Claims {
    uid: uid.into(),
    role: role.into(),
    exp: expiration as usize,
  };

  let header = Header::new(Algorithm::HS512);
  Ok(encode(&header, &claims, &EncodingKey::from_secret(b"secret")).unwrap())
}

pub async fn auth_handler(body: LoginRequest) -> WebResult<impl Reply> {
  // tokio::time::sleep(Duration::from_secs(5)).await;
  println!("Email: {}", body.email);
  // if !body.email.ends_with("ckziu.elodz.edu.pl") {
  //   // return Err(warp::reject::custom(error::Error::WrongCredentialsError));
  //   return Ok(json(
  //     &LoginResponse {
  //       token: None,
  //       message: "Nieprawidłowy email".into()
  //     }
  //   ));
  // }

  Ok(json(&LoginResponse {
    token: Some(create_jwt(body.email.as_str(), "user").unwrap()),
    message: "Pomyślnie zalogowano".into(),
  }))
}