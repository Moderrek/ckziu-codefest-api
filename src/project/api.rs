use chrono::Utc;
use log::{info, warn};
use sqlx::PgPool;
use uuid::Uuid;
use warp::{reject, Reply};
use warp::reply::json;

use project::db;

use crate::{current_millis, error, project, WebResult};
use crate::project::models::Project;
use crate::project::responses::PostProjectRequest;

// GET v1/projects/USER_NAME/PROJECT_NAME
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

// POST v1/project/create
pub async fn create_project(user_uid: Option<Uuid>, req: PostProjectRequest, db_pool: PgPool) -> WebResult<impl Reply> {
  if user_uid.is_none() {
    return Err(reject::custom(error::Error::Unauthorized));
  }
  let owner_id = user_uid.unwrap();

  let project_uid = Uuid::new_v4();

  let project = Project {
    id: project_uid,
    name: req.name.clone(),
    display_name: req.display_name,
    owner_id,
    private: false,
    description: None,
    likes: 0,
    created_at: Utc::now(),
    updated_at: Utc::now(),
  };

  let create_start = current_millis();
  match db::create_project(&project, &db_pool).await {
    Ok(_) => {
      info!("Created new project {}({}) in {}ms", &req.name, &project_uid, current_millis() - create_start);
      Ok("Created")
    }
    Err(err) => {
      warn!("Failed to create project: {}", err);
      Err(reject::custom(error::Error::ServerProblem))
    }
  }
}