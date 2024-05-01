use log::info;
use log::warn;
use sqlx::PgPool;
use uuid::Uuid;
use warp::{reject, Reply};
use warp::reply::json;

use crate::{current_millis, error, WebResult};
use crate::user::db;
use crate::user::responses::{UpdateBioBody, UpdateBioResponse, UpdateDisplayNameBody};

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

// v1/update/user/bio
pub async fn update_bio(uid: Option<Uuid>, body: UpdateBioBody, db_pool: PgPool) -> WebResult<impl Reply> {
  if uid.is_none() {
    return Err(reject::custom(error::Error::Unauthorized));
  }
  let id = uid.unwrap();
  let bio: String = body.bio
    .trim_start()
    .trim_end()
    .into();

  if bio.len() > 100 {
    return Ok(json(&UpdateBioResponse {
      success: false,
      message: "Biografia nie może przekraczać 100 znaków..".into(),
    }));
  }

  let query_start = current_millis();
  match db::update_bio(&id, &db_pool, &body.bio).await {
    Ok(..) => {
      info!("Performed update bio ({}) in {}ms", &id, current_millis() - query_start);
      Ok(json(&UpdateBioResponse {
        success: true,
        message: "Pomyślnie zaktualizowano biografię.".into(),
      }))
    }
    Err(err) => {
      warn!("Failed to update user ({}) bio: {}", &id, err);
      Err(reject::custom(error::Error::ServerProblem))
    }
  }
}

// v1/update/user/displayname
pub async fn update_displayname(uid: Option<Uuid>, body: UpdateDisplayNameBody, db_pool: PgPool) -> WebResult<impl Reply> {
  if uid.is_none() {
    return Err(reject::custom(error::Error::Unauthorized));
  }
  let id = uid.unwrap();

  let displayname: String = body.displayname
    .trim_start()
    .trim_end()
    .into();

  let len = displayname.chars().count();
  if len < 3 {
    return Ok(json(&UpdateBioResponse {
      success: false,
      message: "Wyświetlana nazwa musi posiadać conajmniej 3 znaki.".into(),
    }));
  }
  if len > 30 {
    return Ok(json(&UpdateBioResponse {
      success: false,
      message: "Wyświetlana nazwa nie może przekraczać 30 znaków.".into(),
    }));
  }

  let query_start = current_millis();
  match db::update_display_name(&id, &db_pool, &body.displayname).await {
    Ok(..) => {
      info!("Performed update displayname ({}) in {}ms", &id, current_millis() - query_start);
      Ok(json(&UpdateBioResponse {
        success: true,
        message: "Pomyślnie zaktualizowano wyświetlaną nazwę.".into(),
      }))
    }
    Err(err) => {
      warn!("Failed to update user ({}) displayname: {}", &id, err);
      Err(reject::custom(error::Error::ServerProblem))
    }
  }
}