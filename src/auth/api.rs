use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use chrono::{Duration, Utc};
use jsonwebtoken::EncodingKey;
use log::{debug, info, warn};
use sqlx::PgPool;
use tokio::sync::RwLock;
use uuid::Uuid;
use warp::{reject, Reply};
use warp::reply::json;

use crate::{auth, current_millis, OTPCodes, WebResult};
use crate::auth::db;
use crate::auth::jwt::create_jwt;
use crate::auth::models::AuthUser;
use crate::auth::otp::Otp;
use crate::auth::password::password_verify;
use crate::auth::req::{InfoResponse, LoginCredentialsBody, LoginResponse, OTPRequest, OTPResponse, PreLoginBody, PreLoginResponse, RegisterRequest, RegisterResponse};
use crate::mail::send_otp_code;
use crate::user::models::User;
use crate::utils::addr_to_string;

// v1/auth/prelogin
pub async fn prelogin(addr: Option<SocketAddr>, db: PgPool, body: PreLoginBody) -> WebResult<impl Reply> {
  match db::is_user_exists(&body.login, &db).await {
    Ok(exists) => {
      if exists {
        info!("Peer {} (using {}) received user is registered.", addr_to_string(&addr), &body.login);
        return Ok(json(&PreLoginResponse {
          can_login: true,
          message: "Użytkownik może się zalogować za pomocą hasła.".to_string(),
          status: "200".to_string(),
        }));
      }
      info!("Peer {} (using {}) received user is NOT registered.", addr_to_string(&addr), &body.login);
      Ok(json(&PreLoginResponse {
        can_login: false,
        message: "Użytkownik jest niezarejestrowany.".to_string(),
        status: "404".to_string(),
      }))
    }
    Err(err) => {
      warn!("Peer {} (using {}) cannot prelogin: {}", addr_to_string(&addr), &body.login, err);
      Err(reject::custom(crate::error::Error::ServerProblem))
    }
  }
}

// v1/auth/login/credentials
pub async fn login_credentials(addr: Option<SocketAddr>, db: PgPool, key: EncodingKey, body: LoginCredentialsBody) -> WebResult<impl Reply> {
  let start = Utc::now().timestamp_millis();
  let data = match db::get_user_password_uuid(&body.login, &db).await {
    Ok(exists) => exists,
    Err(err) => {
      warn!("Failed to check is user exists: {}", err);
      return Err(reject::custom(crate::error::Error::ServerProblem));
    }
  };
  info!("Queried login credentials in {}ms", Utc::now().timestamp_millis() - start);

  // User Not Found
  if data.is_none() {
    return Ok(json(&LoginResponse {
      token: None,
      name: None,
      uuid: None,
    }));
  }

  let (password, uuid, name) = data.unwrap();

  // Verify password
  let authorized = match password_verify(&body.password, &password) {
    Ok(authorized) => authorized,
    Err(err) => {
      warn!("Failed to verify password: {}", err);
      return Err(reject::custom(crate::error::Error::ServerProblem));
    }
  };

  if !authorized {
    info!("The {} tried to authorize {}({})", match addr { Some(addr) => addr.to_string(), None => "Unknown".to_string() }, &body.login, uuid);
    return Ok(json(&LoginResponse {
      token: None,
      name: None,
      uuid: None,
    }));
  }

  let token = match create_jwt(uuid, &key) {
    Ok(token) => token,
    Err(err) => {
      warn!("Failed to create authorization token: {}", err);
      return Err(reject::custom(crate::error::Error::ServerProblem));
    }
  };

  info!("Performed login in {}ms", Utc::now().timestamp_millis() - start);

  Ok(json(&LoginResponse {
    token: Some(token),
    name: Some(name),
    uuid: Some(uuid.to_string()),
  }))
}

pub async fn info(userid: Option<Uuid>, db: PgPool) -> WebResult<impl Reply> {
  // Unauthorized
  if userid.is_none() {
    return Err(reject::custom(crate::error::Error::Unauthorized));
  }

  // Authorized
  match crate::user::db::get_info(&userid.unwrap(), &db).await {
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

fn is_password_valid(password: &str) -> bool {
  let mut has_whitespace = false;
  let mut has_upper = false;
  let mut has_lower = false;

  for c in password.chars() {
    has_whitespace |= c.is_whitespace();
    has_lower |= c.is_lowercase();
    has_upper |= c.is_uppercase();
  }

  !has_whitespace && has_upper && has_lower && password.len() >= 8
}

fn is_name_valid(name: &str) -> bool {
  let mut has_whitespace = false;
  let mut has_upper = false;
  let mut has_lower = false;

  for c in name.chars() {
    has_whitespace |= c.is_whitespace();
    has_lower |= c.is_lowercase();
    has_upper |= c.is_uppercase();
  }

  !has_whitespace && !has_upper && has_lower && name.len() >= 3 && name.len() <= 48 && !name.starts_with('-') && !name.ends_with('-')
}

fn is_mail_valid(_mail: &str) -> bool {
  true
}

const CKZIU_MAIL_DOMAIN: &str = "ckziu.elodz.edu.pl";

// v1/auth/otp
pub async fn auth_otp_handler(addr: Option<SocketAddr>, body: OTPRequest, otp_codes: OTPCodes) -> WebResult<impl Reply> {
  // Validate
  if !body.email.ends_with(CKZIU_MAIL_DOMAIN) {
    info!("Peer {} (using {}) tried to receive OTP. Illegal mail.", addr_to_string(&addr), &body.email);
    return Ok(json(&OTPResponse {
      success: false,
      message: "Nielegalny mail.".into(),
    }));
  }

  let mail = body.email;

  let otp = Otp::new_expirable_code(6, Duration::seconds(60));

  // Async save code in a pair with email
  otp_codes.write().await.insert(
    mail.clone(),
    otp.clone(),
  );

  info!("Peer {} (using {}) received OTP code [{}].", addr_to_string(&addr), &mail, &otp.code);

  // OTP will be sent later in a separate async task. Moves `mail`
  tokio::spawn(async move {
    send_otp_code(otp.code, mail);
  });


  Ok(json(&OTPResponse {
    success: true,
    message: "Pomyślnie wysłano kod jednorazowej autoryzacji".into(),
  }))
}

// v1/auth/register
pub async fn register(addr: Option<SocketAddr>, otp_codes: Arc<RwLock<HashMap<String, Otp>>>, key: EncodingKey, db: PgPool, body: RegisterRequest) -> WebResult<impl Reply> {
  debug!("Peer {} (using {}) trying to register new user '{}' with mail '{}', OTP '{}'", addr_to_string(&addr), &body.email, &body.name, &body.email, &body.otp);

  // Validation
  if !is_name_valid(body.name.as_str()) {
    info!("Peer {} (using {}) failed to register. Illegal name.", addr_to_string(&addr), &body.password);
    return Ok(json(&RegisterResponse {
      success: false,
      message: "Nielegalna nazwa.".into(),
      name: None,
      token: None,
    }));
  }
  if !is_mail_valid(body.email.as_str()) {
    info!("Peer {} (using {}) failed to register. Illegal mail.", addr_to_string(&addr), &body.password);
    return Ok(json(&RegisterResponse {
      success: false,
      message: "Nielegalny mail.".into(),
      name: None,
      token: None,
    }));
  }
  if !is_password_valid(body.password.as_str()) {
    info!("Peer {} (using {}) failed to register. Illegal password.", addr_to_string(&addr), &body.password);
    return Ok(json(&RegisterResponse {
      success: false,
      message: "Nielegalne hasło.".into(),
      name: None,
      token: None,
    }));
  }

  // Check OTP Code
  match otp_codes.clone().read().await.get(&body.email) {
    None => {
      // No matching OTP
      info!("Peer {} (using {}) failed to register. No matching OTP for that mail.", addr_to_string(&addr), &body.email);
      return Ok(json(&RegisterResponse {
        success: false,
        name: None,
        token: None,
        message: "Nieprawidłowy kod.".into(),
      }));
    }
    Some(otp) => {
      if otp.is_expired() {
        info!("Peer {} (using {}) failed to register. Tried expired OTP.", addr_to_string(&addr), &body.email);
        // Remove code later async
        let to_remove = body.email.clone();
        tokio::spawn(async move {
          otp_codes.clone().write().await.remove(&to_remove);
        });
        return Ok(json(&RegisterResponse {
          success: false,
          name: None,
          token: None,
          message: "Nieprawidłowy kod.".into(),
        }));
      }

      // Invalid OTP
      if otp.check(&body.otp) {
        info!("Peer {} (using {}) failed to register. Tried invalid OTP.", addr_to_string(&addr), &body.email);
        return Ok(json(&RegisterResponse {
          success: false,
          name: None,
          token: None,
          message: "Nieprawidłowy kod.".into(),
        }));
      }
      // Success. Remove code later async
      let to_remove = body.email.clone();
      tokio::spawn(async move {
        otp_codes.clone().write().await.remove(&to_remove);
      });
    }
  };

  // Check exists for faster performance
  match db::is_user_exists(&body.email, &db).await {
    Ok(exists) => {
      if exists {
        info!("{} failed to register cause user exists '{}'", addr_to_string(&addr), &body.email);
        return Ok(json(&RegisterResponse {
          success: false,
          name: None,
          token: None,
          message: "Użytkownik już istnieje.".into(),
        }));
      }
      // user dont exists
    }
    Err(err) => {
      // db failed
      warn!("Failed to check is user exists: {}", err);
      return Err(reject::custom(crate::error::Error::ServerProblem));
    }
  }

  // Hash password
  let hash_start = current_millis();
  let hashed_password = auth::password::password_hash(&body.password.trim().to_string()).unwrap();
  info!("Hashed password in {}ms", current_millis() - hash_start);

  // Create user data
  let id = Uuid::new_v4();
  let display_name = body.name.clone().replace('-', " ");

  let user = User {
    name: body.name.trim().to_lowercase().to_string(),
    display_name,
    id,
    bio: None,
    created_at: Utc::now(),
    updated_at: Utc::now(),
    flags: 0,
  };

  let auth_user = AuthUser {
    id,
    mail: body.email.trim().to_lowercase(),
    password: hashed_password,
  };

  info!("Creating new user with id {}", &id);

  let db_start = Utc::now().timestamp_millis();
  if let Err(err) = db::register_user(&auth_user, &user, &db).await {
    warn!("Cannot perform register user {}", err);
    return Ok(json(&RegisterResponse {
      success: false,
      name: None,
      token: None,
      message: "Nie udało się zarejestrować. Wystąpił problem serwera.".into(),
    }));
  }
  info!("Success register query for {} in {}ms", &user.name, Utc::now().timestamp_millis() - db_start);

  // Create auth session
  info!("Creating session for {}", &user.name);
  let session_token = match create_jwt(id, &key) {
    Ok(token) => token,
    Err(err) => {
      warn!("Failed to create JWT: {}", err);
      return Ok(json(&RegisterResponse {
        success: false,
        message: "Serwer nie mógł stworzyć sesji.".into(),
        name: None,
        token: None,
      }));
    }
  };

  Ok(json(&RegisterResponse {
    success: true,
    name: Some(body.name),
    token: Some(session_token),
    message: "Pomyślnie zarejestrowano nowe konto i utworzono sesje autoryzacji.".into(),
  }))
}
