use std::net::SocketAddr;
use std::sync::Arc;

use chrono::{Duration, Utc};
use jsonwebtoken::EncodingKey;
use log::{debug, info, warn};
use sqlx::PgPool;
use uuid::Uuid;
use warp::{reject, Reply};
use warp::reply::json;

use crate::{auth, error, user, utils, WebResult};
use crate::auth::db;
use crate::auth::jwt::create_jwt;
use crate::auth::models::AuthUser;
use crate::auth::otp::{Otp, OtpCodes};
use crate::auth::password::password_verify;
use crate::auth::req::{InfoResponse, LoginCredentialsBody, LoginResponse, OTPRequest, OTPResponse, PreLoginBody, PreLoginResponse, RegisterRequest, RegisterResponse};
use crate::error::Error;
use crate::mail::send_otp_code;
use crate::prelude::{web_err, web_json};
use crate::user::models::User;
use crate::utils::{addr_to_string, current_millis};

// POST v1/auth/prelogin
pub async fn prelogin(addr: Option<SocketAddr>, db_pool: PgPool, body: PreLoginBody) -> WebResult<impl Reply> {
  let login = body.login
    .trim()
    .to_string();

  match db::is_user_exists(&login, &login, &db_pool).await {
    Ok(is_registered) => {
      if is_registered {
        info!("Peer {} (using {}) received user is registered.", addr_to_string(&addr), &body.login);
        return web_json(&PreLoginResponse {
          can_login: true,
          message: "Użytkownik może się zalogować za pomocą hasła.".to_string(),
          status: "200".to_string(),
        });
      }
      info!("Peer {} (using {}) received user is NOT registered.", addr_to_string(&addr), &body.login);
      web_json(&PreLoginResponse {
        can_login: false,
        message: "Użytkownik jest niezarejestrowany.".to_string(),
        status: "404".to_string(),
      })
    }
    Err(err) => {
      // Database failed
      warn!("Peer {} (using {}) cannot prelogin: {}", addr_to_string(&addr), &body.login, err);
      web_err(Error::ServerProblem)
    }
  }
}

// POST v1/auth/login/credentials
pub async fn login_credentials(addr: Option<SocketAddr>, db: PgPool, key: Arc<EncodingKey>, body: LoginCredentialsBody) -> WebResult<impl Reply> {
  let login = body.login
    .trim()
    .to_string();

  let data = match db::get_user_password_uuid(&login, &db).await {
    Ok(data) => data,
    Err(err) => {
      warn!("Database failed to get user credentials: {err}");
      return web_err(Error::ServerProblem);
    }
  };

  if data.is_none() {
    // User Not Found
    return web_json(&LoginResponse {
      token: None,
      name: None,
      uuid: None,
    });
  }

  let (password, uuid, name) = data.unwrap();

  // Verify password
  let is_authorized = match password_verify(&body.password, &password) {
    Ok(authorized) => authorized,
    Err(err) => {
      warn!("Authentication failed to verify password: {err}");
      return web_err(Error::ServerProblem);
    }
  };

  if !is_authorized {
    info!("The {} tried to authorize {}({})", addr_to_string(&addr), &body.login, uuid);
    return web_json(&LoginResponse {
      token: None,
      name: None,
      uuid: None,
    });
  }

  let token = match create_jwt(uuid, &key) {
    Ok(token) => token,
    Err(err) => {
      warn!("JWT Failed to create authorization token: {err}");
      return web_err(Error::ServerProblem);
    }
  };

  web_json(&LoginResponse {
    token: Some(token),
    name: Some(name),
    uuid: Some(uuid.to_string()),
  })
}

// GET v1/auth/info
pub async fn info(user_uid: Option<Uuid>, db_pool: PgPool) -> WebResult<impl Reply> {
  // Reject unauthorized. No auth header
  if user_uid.is_none() {
    return web_json(&InfoResponse {
      authorized: false,
      name: None,
    });
  }

  // Authorized
  match user::db::get_info(&user_uid.unwrap(), &db_pool).await {
    Ok(data) => {
      web_json(&InfoResponse {
        authorized: true,
        name: Some(data.0),
      })
    }
    Err(err) => {
      warn!("Database failed to get user: {err}");
      web_err(Error::ServerProblem)
    }
  }
}

// POST v1/auth/otp
pub async fn auth_otp_handler(addr: Option<SocketAddr>, body: OTPRequest, otp_codes: OtpCodes) -> WebResult<impl Reply> {
  // Validate
  let mail = match utils::validate_mail(body.email.clone()) {
    Ok(mail) => mail,
    Err(message) => {
      info!("Peer {} (using {}) tried to receive OTP. Illegal mail. {}", addr_to_string(&addr), &body.email, &message);
      return Ok(json(&OTPResponse {
        success: false,
        message,
      }));
    }
  };

  // Create OTP Code for 8 minutes
  let otp = Otp::new_expirable_code(6, Duration::minutes(8));

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

// POST v1/auth/register
pub async fn register(addr: Option<SocketAddr>, otp_codes: OtpCodes, key: Arc<EncodingKey>, db: PgPool, body: RegisterRequest) -> WebResult<impl Reply> {
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
