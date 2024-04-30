use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use chrono::Utc;
use log::{info, warn};
use sqlx::PgPool;
use tokio::sync::RwLock;
use uuid::Uuid;
use warp::{reject, Reply};
use warp::reply::json;

use crate::{auth, WebResult};
use crate::auth::{db, otp};
use crate::auth::jwt::create_jwt;
use crate::auth::models::AuthUser;
use crate::auth::otp::OTPData;
use crate::auth::password::password_verify;
use crate::auth::req::{InfoResponse, LoginCredentialsBody, LoginResponse, OTPRequest, OTPResponse, PreLoginBody, PreLoginResponse, RegisterRequest, RegisterResponse};
use crate::mail::send_otp_code;
use crate::user::models::User;

// v1/auth/prelogin
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

// v1/auth/login/credentials
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

pub async fn info(userid: Option<Uuid>, db: PgPool) -> WebResult<impl Reply> {
  if userid.is_none() {
    return Err(reject::custom(crate::error::Error::Unauthorized));
  }
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

  !has_whitespace && !has_upper && has_lower && name.len() >= 3 && name.len() <= 48
}

fn is_mail_valid(mail: &str) -> bool {
  true
}

fn is_displayname_valid(displayname: &str) -> bool {
  displayname.len() <= 40
}

fn addr_to_string(addr: &Option<SocketAddr>) -> String {
  match addr {
    Some(addr) => addr.to_string(),
    None => "Unknown".into()
  }
}

// v1/auth/otp
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

  let otp = otp::generate_otp(6);

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

// v1/auth/register
pub async fn register(addr: Option<SocketAddr>, body: RegisterRequest, otp_codes: Arc<RwLock<HashMap<String, OTPData>>>, db: PgPool) -> WebResult<impl Reply> {
  info!("{} trying to register new user '{}' with mail '{}', OTP '{}'", addr_to_string(&addr), &body.name, &body.email, &body.otp);
  // validation
  if !is_name_valid(body.name.as_str()) {
    info!("{} failed to register cause name '{}' is invalid", addr_to_string(&addr), &body.name);
    return Err(reject::custom(crate::error::Error::ParameterProblem));
  }
  if !is_mail_valid(body.email.as_str()) {
    info!("{} failed to register cause name '{}' is invalid", addr_to_string(&addr), &body.email);
    return Err(reject::custom(crate::error::Error::ParameterProblem));
  }
  if !is_password_valid(body.password.as_str()) {
    info!("{} failed to register cause password '{}' is invalid", addr_to_string(&addr), &body.password);
    return Err(reject::custom(crate::error::Error::ParameterProblem));
  }

  // Check OTP Code
  match otp_codes.clone().read().await.get(&body.email.clone()) {
    None => {
      // No matching OTP
      info!("{} failed to register cause no matching otp for this email  '{}'", addr_to_string(&addr), &body.email);
      return Ok(json(&RegisterResponse {
        success: false,
        name: None,
        token: None,
        message: "Nieprawidłowy kod.".into(),
      }));
    }
    Some(otp_data) => {
      // OTP Expired
      if otp_data.expired.timestamp() < Utc::now().timestamp() {
        info!("{} failed to register cause expired otp '{}'", addr_to_string(&addr), &body.email);
        tokio::spawn(async move {
          otp_codes.clone().write().await.remove(&body.email);
        });
        return Ok(json(&RegisterResponse {
          success: false,
          name: None,
          token: None,
          message: "Nieprawidłowy kod.".into(),
        }));
      }

      // Invalid OTP
      if otp_data.code != body.otp {
        info!("{} failed to register cause invalid otp '{}'", addr_to_string(&addr), &body.email);
        return Ok(json(&RegisterResponse {
          success: false,
          name: None,
          token: None,
          message: "Nieprawidłowy kod.".into(),
        }));
      }
      // Success
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
        return Err(reject::custom(crate::error::Error::UserExists));
      }
      // user dont exists
    }
    Err(err) => {
      // db failed
      warn!("Failed to check is user exists: {}", err);
      return Err(reject::custom(crate::error::Error::ServerProblem));
    }
  }

  match db::is_user_exists(&body.name, &db).await {
    Ok(exists) => {
      if exists {
        info!("{} failed to register cause user exists '{}'", addr_to_string(&addr), &body.name);
        return Err(reject::custom(crate::error::Error::UserExists));
      }
    }
    Err(err) => {
      // db failed
      warn!("Failed to check is user exists: {}", err);
      return Err(reject::custom(crate::error::Error::ServerProblem));
    }
  }

  // Hash password
  let hash_start = Utc::now().timestamp_millis();
  let hashed_password = auth::password::password_hash(&body.password.trim().to_string()).unwrap();
  info!("Hashed password in {}ms", Utc::now().timestamp_millis() - hash_start);

  // Create user data
  let id = Uuid::new_v4();
  let display_name = body.name.clone().replace('-', " ");

  let user = User {
    name: body.name.to_lowercase().trim().to_string(),
    display_name,
    id,
    bio: None,
    created_at: Utc::now(),
    updated_at: Utc::now(),
    flags: 0,
  };

  let auth_user = AuthUser {
    id,
    mail: body.email.to_lowercase().trim().to_lowercase().clone(),
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

  // Create session
  info!("Creating session for {}", &user.name);
  let session_token = create_jwt(id).expect("Failed to create JWT");

  Ok(json(&RegisterResponse {
    success: true,
    name: Some(body.name),
    token: Some(session_token),
    message: "Pomyślnie zarejestrowano nowe konto i utworzono sesje autoryzacji.".into(),
  }))
}
