use chrono::Utc;
use log::warn;
use sqlx::PgPool;
use uuid::Uuid;
use warp::{reject, Reply};
use warp::reply::json;

use project::db;

use crate::{error, project, WebResult};
use crate::project::models::Project;
use crate::project::responses::PostProjectRequest;

pub async fn get_project(username: String, project_name: String, db_pool: PgPool) -> WebResult<impl Reply> {
  match db::get_project_by_ownername_projectname(&username, &project_name, &db_pool).await {
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

pub async fn create_project(uid: Option<Uuid>, req: PostProjectRequest, db_pool: PgPool) -> WebResult<impl Reply> {
  if uid.is_none() {
    return Ok("UNAUTHORIZED");
  }
  let uid = uid.unwrap();

  let project_uid = Uuid::new_v4();

  let project = Project {
    id: project_uid,
    name: req.name,
    display_name: req.display_name,
    owner_id: uid,
    private: false,
    description: None,
    likes: 0,
    created_at: Utc::now(),
    updated_at: Utc::now(),
  };

  match db::create_project(&project, &db_pool).await {
    Ok(_) => {}
    Err(err) => {
      warn!("{}", err);
    }
  }

  Ok("AUTHORIZED")
}