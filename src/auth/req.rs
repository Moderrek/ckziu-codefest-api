use serde::{Deserialize, Serialize};

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

#[derive(Serialize)]
pub struct InfoResponse {
  pub authorized: bool,
  pub name: Option<String>,
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
  pub message: String,
  pub name: Option<String>,
  pub token: Option<String>,
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
