use log::warn;
use sqlx::PgPool;
use uuid::Uuid;
use warp::{reject, Reply};
use warp::reply::json;

use project::db;

use crate::{error, project, WebResult};
use crate::project::responses::PostProjectRequest;

pub async fn get_project(username: String, project_name: String, db_pool: PgPool) -> WebResult<impl Reply> {
  match db::get_project(&username, &project_name, &db_pool).await {
    Ok(response) => {
      if let Some(project) = response {
        return Ok(json(&project));
      }
    }
    Err(err) => {
      warn!("Detected server problem @ DB PROJECT GET: {}", err);
      return Err(reject::custom(error::Error::ServerProblem));
    }
  }
  Err(reject::custom(error::Error::ProjectNotFound))
}

pub async fn post_project(uid: Uuid, req: PostProjectRequest, db_pool: PgPool) -> WebResult<impl Reply> {
  Ok(uid.to_string())
}