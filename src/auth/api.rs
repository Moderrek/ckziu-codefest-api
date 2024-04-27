use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use chrono::Utc;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::postgres::PgSeverity::Log;
use tokio::sync::RwLock;
use uuid::Uuid;
use warp::{reject, Reply};
use warp::reply::json;

use crate::{auth, WebResult};
use crate::auth::{create_jwt, db, db_register_user, OTPData, RegisterRequest, RegisterResponse};
use crate::auth::auth::password_verify;
use crate::auth::models::AuthUser;
use crate::user::models::User;

#[derive(Deserialize)]
pub struct ExistsBody {
  pub login: String,
}

#[derive(Deserialize)]
pub struct PreLoginBody {
  pub login: String,
}

#[derive(Deserialize)]
pub struct LoginCredentialsBody {
  pub login: String,
  pub password: String,
}

#[derive(Deserialize)]
pub struct RequestOtpBody {
  pub mail: String,
}

#[derive(Deserialize)]
pub struct LoginOtpBody {
  pub mail: String,
  pub otp: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterBody {
  pub email: String,
  pub otp: String,

  pub name: String,
  pub password: String,
}

#[derive(Serialize)]
pub struct PreLoginResponse {
  pub can_login: bool,
  pub message: String,
  pub status: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
  pub token: Option<String>,
  pub name: Option<String>,
  pub uuid: Option<String>,
}

pub async fn exists(body: ExistsBody, db: PgPool) -> WebResult<impl Reply> {
  match db::is_user_exists(&body.login, &db).await {
    Ok(exists) => {
      Ok(json(&exists))
    }
    Err(err) => {
      warn!("Failed to check is user exists: {}", err);
      Err(reject::custom(crate::error::Error::ServerProblem))
    }
  }
}

pub async fn prelogin(body: PreLoginBody, db: PgPool) -> WebResult<impl Reply> {
  match db::is_user_exists(&body.login, &db).await {
    Ok(exists) => {
      if exists {
        return Ok(json(&PreLoginResponse {
          can_login: true,
          message: "Użytkownik może się zalogować za pomocą hasła.".to_string(),
          status: "200".to_string(),
        }));
      }
      Ok(json(&PreLoginResponse {
        can_login: false,
        message: "Użytkownik jest niezarejestrowany.".to_string(),
        status: "404".to_string(),
      }))
    }
    Err(err) => {
      warn!("Failed to check is user exists: {}", err);
      Err(reject::custom(crate::error::Error::ServerProblem))
    }
  }
}

pub async fn login_credentials(addr: Option<SocketAddr>, body: LoginCredentialsBody, db: PgPool) -> WebResult<impl Reply> {
  let start = Utc::now().timestamp_millis();
  let optional_data = match db::get_user_password_uuid(&body.login, &db).await {
    Ok(exists) => exists,
    Err(err) => {
      warn!("Failed to check is user exists: {}", err);
      return Err(reject::custom(crate::error::Error::ServerProblem));
    }
  };
  info!("Query \"Login Credentials\" in {}ms", Utc::now().timestamp_millis() - start);

  if optional_data.is_none() {
    return Ok(json(&LoginResponse {
      token: None,
      name: None,
      uuid: None,
    }));
  }

  let data = optional_data.unwrap();

  let authorized = match password_verify(&body.password, &data.0) {
    Ok(authorized) => authorized,
    Err(err) => {
      warn!("Failed to verify password: {}", err);
      return Err(reject::custom(crate::error::Error::ServerProblem));
    }
  };

  if !authorized {
    info!("The {} tried to authorize {}({})", match addr { Some(addr) => addr.to_string(), None => "Unknown".to_string() }, &body.login, data.1);
    return Ok(json(&LoginResponse {
      token: None,
      name: None,
      uuid: None,
    }));
  }

  let token = match create_jwt(data.1) {
    Ok(token) => token,
    Err(err) => {
      warn!("Failed to create authorization token: {}", err);
      return Err(reject::custom(crate::error::Error::ServerProblem));
    }
  };

  Ok(json(&LoginResponse {
    token: Some(token),
    name: Some(data.2),
    uuid: Some(data.1.to_string()),
  }))
}

#[derive(Serialize)]
struct InfoResponse {
  name: String,
}

pub async fn info(userid: Uuid, db: PgPool) -> WebResult<impl Reply> {
  match crate::user::db::get_info(&userid, &db).await {
    Ok(info) => {
      Ok(json(&InfoResponse {
        name: info.0
      }))
    }
    Err(err) => {
      warn!("Failed to get info: {}", err);
      Err(reject::custom(crate::error::Error::ServerProblem))
    }
  }
}

pub async fn register(body: RegisterRequest, pool: PgPool) -> WebResult<impl Reply> {
  let id = Uuid::new_v4();

  let user = User {
    name: body.name.to_lowercase().trim().to_string(),
    display_name: body.name.clone(),
    id,
    bio: None,
    created_at: Utc::now(),
    updated_at: Utc::now(),
    flags: 0,
  };

  let auth_user = AuthUser {
    id,
    mail: body.email.to_lowercase().trim().to_lowercase().clone(),
    password: auth::auth::password_hash(&body.password.trim().to_string()).unwrap(),
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
      let token = create_jwt(id).expect("Failed to create JWT");

      Ok(json(&RegisterResponse {
        success: true,
        token: Some(token),
        message: "Pomyślnie utworzono konto".into(),
      }))
    }
  }
}
