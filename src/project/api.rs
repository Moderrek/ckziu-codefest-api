use chrono::Utc;
use log::{info, warn};
use sqlx::PgPool;
use uuid::Uuid;
use warp::{reject, Reply};
use warp::reply::json;

use project::db;

use crate::{current_millis, error, project, WebResult};
use crate::project::models::Project;
use crate::project::responses::PostProjectBody;

use super::{validate_description, validate_display_name, validate_name};
use super::responses::PostProjectResponse;

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
pub async fn create_project(user_uid: Option<Uuid>, body: PostProjectBody, db_pool: PgPool) -> WebResult<impl Reply> {
  // Reject unauthorized
  if user_uid.is_none() {
    return Err(reject::custom(error::Error::Unauthorized));
  }
  // Validate
  let project_name = match validate_name(body.name) {
    Ok(name) => name,
    Err(message) => {
      return Ok(json(&PostProjectResponse {
        success: false,
        created: false,
        message,
      }));
    }
  };
  let project_display_name = match validate_display_name(body.display_name) {
    Ok(display_name) => display_name,
    Err(message) => {
      return Ok(json(&PostProjectResponse {
        success: false,
        created: false,
        message,
      }));
    }
  };
  let project_description = match validate_description(body.description) {
    Ok(description) => description,
    Err(message) => {
      return Ok(json(&PostProjectResponse {
        success: false,
        created: false,
        message,
      }));
    }
  };

  let owner_id = user_uid.unwrap();

  match db::has_project_by_id(&owner_id, &project_name, &db_pool).await {
    Ok(exists) => {
      if exists {
        return Ok(json(&PostProjectResponse {
          success: false,
          created: false,
          message: "Projekt już istnieje!".into(),
        }));
      }
    }
    Err(err) => {
      warn!("Failed to check exist project: {err}");
      return Ok(json(&PostProjectResponse {
        success: false,
        created: false,
        message: "Nie udało się sprawdzić dostępności projektu.".into(),
      }));
    }
  }

  let project_uid = Uuid::new_v4();

  // Create data
  let project = Project {
    id: project_uid,
    name: project_name.clone(),
    display_name: project_display_name.clone(),
    owner_id,
    private: false,
    description: project_description,
    likes: 0,
    created_at: Utc::now(),
    updated_at: Utc::now(),
  };

  // Upload to database
  let create_start = current_millis();
  match db::create_project(&project, &db_pool).await {
    Ok(_) => {
      info!("Created new project {}({}) owner {} in {}ms", &project_name, &project_uid, &owner_id, current_millis() - create_start);
      Ok(json(&PostProjectResponse {
        success: true,
        created: true,
        message: "Pomyślnie utworzono.".into(),
      }))
    }
    Err(err) => {
      warn!("Failed to create project: {}", err);
      Err(reject::custom(error::Error::ServerProblem))
    }
  }
}