use std::convert::Infallible;

use serde::Serialize;
use thiserror::Error;
use warp::{http::StatusCode, Rejection, Reply};

#[derive(Error, Debug)]
pub enum Error {
  #[error("wrong credentials")]
  WrongCredentials,
  #[error("jwt token not valid")]
  JWTToken,
  #[error("jwt token creation error")]
  JWTTokenCreation,
  #[error("no auth header")]
  NoAuthHeader,
  #[error("invalid auth header")]
  InvalidAuthHeader,
  #[error("no permission")]
  NoPermission,
  #[error("Cannot find file")]
  CannotFindFile,
  #[error("Not Found")]
  NotFound,
  #[error("User Not Found")]
  UserNotFound,
  #[error("Project Not Found")]
  ProjectNotFound,
  #[error("Server Problem")]
  ServerProblem,
  #[error("Parameter Problem")]
  ParameterProblem,
  #[error("User Exists")]
  UserExists,
  #[error("Unauthorized")]
  Unauthorized,
}

#[derive(Serialize, Debug)]
struct ErrorResponse {
  success: bool,
  message: String,
  status: String,
}

impl warp::reject::Reject for Error {}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
  let (code, message) = if err.is_not_found() {
    (StatusCode::NOT_FOUND, "Not Found".to_string())
  } else if let Some(e) = err.find::<Error>() {
    match e {
      Error::WrongCredentials => (StatusCode::FORBIDDEN, e.to_string()),
      Error::NoPermission => (StatusCode::UNAUTHORIZED, e.to_string()),
      Error::JWTToken => (StatusCode::UNAUTHORIZED, e.to_string()),
      Error::Unauthorized => (StatusCode::UNAUTHORIZED, e.to_string()),
      Error::JWTTokenCreation => (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal Server Error".to_string(),
      ),
      Error::ServerProblem => (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal Server Error".to_string(),
      ),
      _ => (StatusCode::BAD_REQUEST, e.to_string()),
    }
  } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
    (
      StatusCode::METHOD_NOT_ALLOWED,
      "Method Not Allowed".to_string(),
    )
  } else {
    eprintln!("unhandled error: {:?}", err);
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      "Internal Server Error".to_string(),
    )
  };

  let json = warp::reply::json(&ErrorResponse {
    success: false,
    status: code.to_string(),
    message,
  });

  Ok(warp::reply::with_status(json, code))
}
