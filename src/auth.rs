use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use jsonwebtoken::{Algorithm, encode, EncodingKey, Header};
use log::{info, warn};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tokio::sync::RwLock;
use uuid::Uuid;
use warp::Reply;
use warp::reply::json;

use crate::mail::send_otp_code;
use crate::users::CodeFestUser;
use crate::WebResult;

#[derive(Debug, Deserialize, Serialize, FromRow)]
struct AuthUser {
  name: String,
  mail: String,
  id: Uuid,
  password: String,
}

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
  pub password: String,
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
  Ok(encode(
    &header,
    &claims,
    &EncodingKey::from_secret(dotenv!("TOKEN_SECRET").as_bytes()),
  )
    .unwrap())
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

pub async fn auth_otp_handler(
  body: OTPRequest,
  otp_codes: Arc<RwLock<HashMap<String, OTPData>>>,
) -> WebResult<impl Reply> {
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

  otp_codes.write().await.insert(
    body.email.clone(),
    OTPData {
      code: otp.clone(),
      expired: expiration,
    },
  );

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

pub async fn auth_login_handler(
  body: LoginRequest,
  otp_codes: Arc<RwLock<HashMap<String, OTPData>>>,
) -> WebResult<impl Reply> {
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

pub fn hash_password(password: String) -> String {
  password
}

async fn db_register_user(auth_user: &AuthUser, user: &CodeFestUser, pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
  let mut transaction = pool.begin().await?;

  let auth_query = "INSERT INTO auth (name, mail, id, password) VALUES ($1, $2, $3, $4)";

  sqlx::query(auth_query)
    .bind(&auth_user.name)
    .bind(&auth_user.mail)
    .bind(auth_user.id)
    .bind(&auth_user.password)
    .execute(&mut *transaction)
    .await?;

  let query = "INSERT INTO users (name, display_name, id, bio, created_at, updated_at, flags) VALUES ($1, $2, $3, $4, $5, $6, $7)";

  sqlx::query(query)
    .bind(&user.name)
    .bind(&user.display_name)
    .bind(user.id)
    .bind(&user.bio)
    .bind(user.created_at)
    .bind(user.updated_at)
    .bind(user.flags)
    .execute(&mut *transaction)
    .await?;

  transaction.commit().await?;

  Ok(())
}

async fn db_find_user(selector: &String, pool: &PgPool) -> Result<Option<AuthUser>, Box<dyn std::error::Error>> {
  let mut transaction = pool.begin().await?;

  let find_query = "SELECT * FROM auth WHERE name = $1 or mail = $1";

  let result: Option<AuthUser> = sqlx::query_as(find_query)
    .bind(selector)
    .fetch_optional(&mut *transaction)
    .await?;

  transaction.commit().await?;

  Ok(result)
}

pub async fn auth_exists_handler(selector: String, pool: PgPool) -> WebResult<impl Reply> {
  let selector = selector.to_lowercase();

  match db_find_user(&selector, &pool).await {
    Ok(option_user) => {
      if let Some(user) = option_user {
        return Ok(user.id.to_string());
      }
      Ok("Not Found".into())
    }
    Err(err) => {
      warn!("Auth exists: {}", err);
      Ok("Error".into())
    }
  }
}

pub async fn auth_register_handler(body: RegisterRequest, pool: PgPool) -> WebResult<impl Reply> {
  let id = Uuid::new_v4();

  let user = CodeFestUser {
    name: body.name.to_lowercase().clone(),
    display_name: body.name.clone(),
    id,
    bio: None,
    created_at: Utc::now(),
    updated_at: Utc::now(),
    flags: 0,
  };

  let auth_user = AuthUser {
    name: body.name.to_lowercase().clone(),
    id,
    mail: body.email.to_lowercase().clone(),
    password: hash_password(body.password),
  };

  match db_register_user(&auth_user, &user, &pool).await {
    Err(err) => {
      warn!("Cannot perform register user {}", err);
      Ok(json(&RegisterResponse {
        success: false,
        token: None,
        message: "Nie udało się zarejestrować".into(),
      }))
    }
    _ => {
      info!("New user {}", user.name.clone());
      let token = create_jwt(user.id.to_string().as_str(), "user").expect("Failed to create JWT");

      Ok(json(&RegisterResponse {
        success: true,
        token: Some(token),
        message: "Pomyślnie utworzono konto".into(),
      }))
    }
  }
}
