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

use crate::{auth, error, user, utils, OTPCodes, WebResult};
use crate::auth::db;
use crate::auth::jwt::create_jwt;
use crate::auth::models::AuthUser;
use crate::auth::otp::Otp;
use crate::auth::password::password_verify;
use crate::auth::req::{InfoResponse, LoginCredentialsBody, LoginResponse, OTPRequest, OTPResponse, PreLoginBody, PreLoginResponse, RegisterRequest, RegisterResponse};
use crate::mail::send_otp_code;
use crate::user::models::User;
use crate::utils::{addr_to_string, current_millis};

// v1/auth/prelogin
pub async fn prelogin(addr: Option<SocketAddr>, db: PgPool, body: PreLoginBody) -> WebResult<impl Reply> {
  let login = body.login.trim().to_string();

  match db::is_user_exists(&login.clone(), &login.clone(), &db).await {
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
      Err(reject::custom(error::Error::ServerProblem))
    }
  }
}

// v1/auth/login/credentials
pub async fn login_credentials(addr: Option<SocketAddr>, db: PgPool, key: EncodingKey, body: LoginCredentialsBody) -> WebResult<impl Reply> {
  let login = body.login.trim().into();
  let start = current_millis();
  let data = match db::get_user_password_uuid(&login, &db).await {
    Ok(exists) => exists,
    Err(err) => {
      warn!("Failed to check is user exists: {}", err);
      return Err(reject::custom(error::Error::ServerProblem));
    }
  };
  info!("Queried login credentials in {}ms", current_millis() - start);

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
      return Err(reject::custom(error::Error::ServerProblem));
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
      return Err(reject::custom(error::Error::ServerProblem));
    }
  };

  info!("Performed login in {}ms", current_millis() - start);

  Ok(json(&LoginResponse {
    token: Some(token),
    name: Some(name),
    uuid: Some(uuid.to_string()),
  }))
}

pub async fn info(user_uid: Option<Uuid>, db: PgPool) -> WebResult<impl Reply> {
  // Unauthorized
  if user_uid.is_none() {
    return Ok(json(&InfoResponse {
      authorized: false,
      name: None
    }));
  }
  let user_uid = user_uid.unwrap();

  // Authorized
  match user::db::get_info(&user_uid, &db).await {
    Ok(data) => {
      Ok(json(&InfoResponse {
        authorized: true,
        name: Some(data.0)
      }))
    }
    Err(err) => {
      warn!("Failed to get info: {}", err);
      Err(reject::custom(error::Error::ServerProblem))
    }
  }
}

// v1/auth/otp
pub async fn auth_otp_handler(addr: Option<SocketAddr>, body: OTPRequest, otp_codes: OTPCodes) -> WebResult<impl Reply> {
  // Validate
  let mail = match utils::validate_mail(body.email.clone()) {
    Ok(mail) => mail,
    Err(message) => {
      info!("Peer {} (using {}) tried to receive OTP. Illegal mail. {}", addr_to_string(&addr), &body.email, &message);
      return Ok(json(&OTPResponse {
        success: false,
        message
      }));
    }
  };

  // Create OTP Code for 5 minutes
  let otp = Otp::new_expirable_code(6, Duration::minutes(5));

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
pub async fn register(addr: Option<SocketAddr>, otp_codes: OTPCodes, key: EncodingKey, db: PgPool, body: RegisterRequest) -> WebResult<impl Reply> {
  debug!("Peer {} trying to register new user '{}' with mail '{}', OTP '{}'", addr_to_string(&addr), &body.name, &body.email, &body.otp);

  // Validation
  let name = match utils::validate_name(body.name.clone()) {
    Ok(name) => name,
    Err(message) => {
      info!("Peer {} (using {}) failed to register. Illegal name. {}", addr_to_string(&addr), &body.email, &message);
      return Ok(json(&RegisterResponse {
        success: false,
        message,
        name: None,
        token: None,
      }));
    }
  };
  let display_name = name.clone();
  let mail = match utils::validate_mail(body.email.clone()) {
    Ok(mail) => mail,
    Err(message) => {
      info!("Peer {} (using {}) failed to register. Illegal mail. {}", addr_to_string(&addr), &body.email, &message);
      return Ok(json(&RegisterResponse {
        success: false,
        message,
        name: None,
        token: None,
      }));
    }
  };
  let password = match utils::validate_password(body.password.clone()) {
    Ok(password) => password,
    Err(message) => {
      info!("Peer {} (using {}) failed to register. Illegal password. {}", addr_to_string(&addr), &mail, &message);
      return Ok(json(&RegisterResponse {
        success: false,
        message,
        name: None,
        token: None,
      }));
    }
  };

  // Check OTP Code
  match otp_codes.clone().read().await.get(&mail) {
    None => {
      // No matching OTP
      info!("Peer {} (using {}) failed to register. No matching OTP for that mail.", addr_to_string(&addr), &mail);
      return Ok(json(&RegisterResponse {
        success: false,
        name: None,
        token: None,
        message: "Nieprawidłowy kod OTP.".into(),
      }));
    }
    Some(otp) => {
      if otp.is_expired() {
        info!("Peer {} (using {}) failed to register. Tried expired OTP.", addr_to_string(&addr), &mail);
        // Remove code later async
        let to_remove = mail.clone();
        tokio::spawn(async move {
          otp_codes.clone().write().await.remove(&to_remove);
        });
        return Ok(json(&RegisterResponse {
          success: false,
          name: None,
          token: None,
          message: "Nieprawidłowy kod. Kod wygasł.".into(),
        }));
      }

      // Invalid OTP
      let otp_to_check = body.otp.trim().to_string();
      if !otp.check(&otp_to_check) {
        info!("Peer {} (using {}) failed to register. Tried invalid OTP. '{}' != '{}'", addr_to_string(&addr), &mail, otp.code, otp_to_check);
        return Ok(json(&RegisterResponse {
          success: false,
          name: None,
          token: None,
          message: "Nieprawidłowy kod.".into(),
        }));
      }
      // Success. Remove code later async
      let to_remove = mail.clone();
      tokio::spawn(async move {
        otp_codes.clone().write().await.remove(&to_remove);
      });
    }
  };

  // Check exists for better performance
  match db::is_user_exists(&name, &mail, &db).await {
    Ok(exists) => {
      if exists {
        info!("{} failed to register cause user exists '{}'", addr_to_string(&addr), &mail);
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
      return Err(reject::custom(error::Error::ServerProblem));
    }
  }

  // Hash password
  let hash_start = current_millis();
  let hashed_password = auth::password::password_hash(&password).unwrap();
  info!("Hashed password in {}ms", current_millis() - hash_start);

  // Create user data
  let id = Uuid::new_v4();

  let user = User {
    name: name.clone(),
    display_name: display_name.clone(),
    id,
    bio: None,
    created_at: Utc::now(),
    updated_at: Utc::now(),
    flags: 0,
  };

  let auth_user = AuthUser {
    id,
    mail: mail.clone(),
    password: hashed_password,
  };

  info!("Creating new user with id {}", &id);

  let db_start = current_millis();
  if let Err(err) = db::register_user(&auth_user, &user, &db).await {
    warn!("Cannot perform register user {}", err);
    return Ok(json(&RegisterResponse {
      success: false,
      name: None,
      token: None,
      message: "Nie udało się zarejestrować. Wystąpił problem serwera.".into(),
    }));
  }
  info!("Success register query for {} in {}ms", &name, current_millis() - db_start);

  // Create auth session
  info!("Creating session for {}", &name);
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
    name: Some(name),
    token: Some(session_token),
    message: "Pomyślnie zarejestrowano nowe konto i utworzono sesje autoryzacji.".into(),
  }))
}
