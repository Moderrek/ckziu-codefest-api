use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use jsonwebtoken::{Algorithm, encode, EncodingKey, Header};
use log::info;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use warp::Reply;
use warp::reply::json;

use crate::{WebResult};
use crate::mail::send_otp_code;

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
  uid: String,
  role: String,
  exp: usize,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
  pub email: String,
  pub otp: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
  pub success: bool,
  pub token: Option<String>,
  pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct OTPRequest {
  pub email: String,
}

#[derive(Debug, Serialize)]
pub struct OTPResponse {
  pub message: String,
  pub success: bool,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
  pub email: String,
  pub otp: String,
  pub name: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
  pub success: bool,
  pub token: Option<String>,
  pub message: String,
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
  Ok(encode(&header, &claims, &EncodingKey::from_secret(dotenv!("TOKEN_SECRET").as_bytes())).unwrap())
}

fn generate_otp(length: usize) -> String {
  let digits = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
  let mut buffer = String::with_capacity(length);

  for _ in 0..length {
    buffer.push(*digits.choose(&mut rand::thread_rng()).unwrap());
  }

  buffer
}

#[derive(Debug)]
pub struct OTPData {
  pub code: String,
  pub expired: DateTime<Utc>,
}

pub async fn auth_otp_handler(body: OTPRequest, otp_codes: Arc<RwLock<HashMap<String, OTPData>>>) -> WebResult<impl Reply> {
  println!("OTP -> Email: {}", body.email);
  // if !body.email.ends_with("ckziu.elodz.edu.pl") {
  //   // return Err(warp::reject::custom(error::Error::WrongCredentialsError));
  //   return Ok(json(
  //     &LoginResponse {
  //       token: None,
  //       message: "Nieprawidłowy email".into()
  //     }
  //   ));
  // }

  let otp = generate_otp(6);

  let expiration = Utc::now()
    .checked_add_signed(chrono::Duration::seconds(60))
    .expect("Valid timestamp");

  otp_codes.write().await.insert(body.email.clone(), OTPData {
    code: otp.clone(),
    expired: expiration,
  });

  info!("{} | OTP = {}", body.email, &otp);
  tokio::spawn(async move {
    send_otp_code(otp, body.email);
  });

  info!("Sent OTP Response");

  Ok(json(&OTPResponse {
    message: "Pomyślnie wysłano kod jednorazowej autoryzacji".into(),
    success: true,
  }))
}

pub async fn auth_login_handler(body: LoginRequest, otp_codes: Arc<RwLock<HashMap<String, OTPData>>>) -> WebResult<impl Reply> {
  info!("LOGIN @ {} | with code = {}", &body.email, &body.otp);

  return match otp_codes.clone().read().await.get(&body.email.clone()) {
    None => {
      info!("LOGIN @ {} | No matching code", &body.email);
      Ok(json(&LoginResponse {
        success: false,
        token: None,
        message: "Nieprawidłowy kod".into(),
      }))
    }
    Some(otp_data) => {
      if otp_data.expired.timestamp() < Utc::now().timestamp() {
        info!("LOGIN @ {} | Wygaszony kod!", &body.email);
        tokio::spawn(async move {
          otp_codes.clone().write().await.remove(&body.email);
        });
        return Ok(json(&LoginResponse {
          success: false,
          token: None,
          message: "Nieprawidłowy kod".into(),
        }));
      }
      if otp_data.code == body.otp {
        info!("LOGIN @ {} | Prawidłowy kod | Zalogowano", &body.email);
        let jwt = create_jwt(body.email.as_str(), "user").unwrap();
        tokio::spawn(async move {
          otp_codes.clone().write().await.remove(&body.email);
        });
        Ok(json(&LoginResponse {
          success: true,
          token: Some(jwt),
          message: "Pomyślnie zalogowano".into(),
        }))
      } else {
        info!("LOGIN @ {} | Zły kod!", &body.email);
        Ok(json(&LoginResponse {
          success: false,
          token: None,
          message: "Nieprawidłowy kod".into(),
        }))
      }
    }
  };
}

pub async fn auth_register_handler(body: RegisterRequest) -> WebResult<impl Reply> {
  Ok(json(&RegisterResponse {
    success: false,
    token: None,
    message: "Nie udało się zarejestrować".into(),
  }))
}