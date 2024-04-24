use crypto::common::typenum::op;
use log::warn;
use sqlx::PgPool;
use warp::{reject, Reply};
use warp::reply::json;

use crate::{error, project, WebResult};
use crate::user::db;
use crate::user::responses::ProfileResponse;

pub async fn get_user(username: String, db_pool: PgPool) -> WebResult<impl Reply> {
  match db::get_user(&username, &db_pool).await {
    Ok(response) => {
      if let Some(user) = response {
        return Ok(json(&user));
      }
    }
    Err(err) => {
      warn!("Detected server problem @ DB USER GET: {}", err);
      return Err(reject::custom(error::Error::ServerProblem));
    }
  }
  Err(reject::custom(error::Error::UserNotFound))
}

pub async fn get_profile(username: String, db_pool: PgPool) -> WebResult<impl Reply> {
  match db::get_profile(&username, &db_pool).await {
    Ok(response) => {
      if let Some(profile) = response {
        return Ok(json(&profile));
      }
    }
    Err(err) => {
      warn!("Detected server problem @ DB PROFILE GET: {}", err);
      return Err(reject::custom(error::Error::ServerProblem));
    }
  }
  Err(reject::custom(error::Error::UserNotFound))
}
