use jsonwebtoken::{Algorithm, decode, DecodingKey, Validation};
use log::warn;
use uuid::Uuid;
use warp::{Filter, Rejection};
use warp::header::headers_cloned;
use warp::http::{HeaderMap, HeaderValue};
use warp::http::header::AUTHORIZATION;

use crate::auth::jwt::Claims;
use crate::{error, WebResult};

const BEARER: &str = "Bearer ";

pub fn with_auth() -> impl Filter<Extract=(Option<Uuid>, ), Error=Rejection> + Clone {
  headers_cloned()
    .map(move |headers: HeaderMap<HeaderValue>| (headers))
    .and_then(authorize)
}

async fn authorize(headers: HeaderMap<HeaderValue>) -> WebResult<Option<Uuid>> {
  match jwt_from_header(&headers) {
    Ok(jwt) => {
      let decoded = match decode::<Claims>(
        &jwt,
        &DecodingKey::from_secret(dotenv!("TOKEN_SECRET").as_bytes()),
        &Validation::new(Algorithm::HS512),
      ) {
        Ok(decoded) => decoded,
        Err(err) => {
          warn!("Encountered error {}", err);
          return Ok(None);
        }
      };

      let uid = decoded.claims.uuid;

      Ok(Some(uid))
    }
    Err(err) => {
      warn!("Encountered error {}", err);
      Ok(None)
    }
  }
}

fn jwt_from_header(headers: &HeaderMap<HeaderValue>) -> Result<String, error::Error> {
  let header = match headers.get(AUTHORIZATION) {
    Some(value) => value,
    None => return Err(error::Error::NoAuthHeader),
  };
  let auth_header = match std::str::from_utf8(header.as_bytes()) {
    Ok(value) => value,
    Err(_) => return Err(error::Error::NoAuthHeader),
  };
  if !auth_header.starts_with(BEARER) {
    return Err(error::Error::InvalidAuthHeader);
  }
  Ok(auth_header.trim_start_matches(BEARER).to_owned())
}
