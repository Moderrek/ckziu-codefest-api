use chrono::{DateTime, Utc};
use chrono::serde::ts_milliseconds;
use log::{info, warn};
use reqwest::StatusCode;
use serde::{de, Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::Execute;
use sqlx::PgPool;
use sqlx::Postgres;
use sqlx::QueryBuilder;
use uuid::Uuid;
use warp::{reject, Reply};
use warp::reply::json;

use project::db;

use crate::{current_millis, error, project, WebResult};
use crate::project::models::Project;
use crate::project::responses::PostProjectBody;
use crate::user::api::is_authorized;

use super::{validate_description, validate_display_name, validate_name};
use super::responses::PostProjectResponse;

#[derive(Serialize, FromRow)]
pub struct FullProjectResponse {
  pub id: Uuid,
  pub name: String,
  pub display_name: String,
  pub private: bool,
  pub owner_id: Uuid,
  pub owner_name: String,
  pub url: String,
  pub description: Option<String>,
  pub github_url: Option<String>,
  pub website_url: Option<String>,
  #[serde(with = "ts_milliseconds")]
  pub created_at: DateTime<Utc>,
  #[serde(with = "ts_milliseconds")]
  pub updated_at: DateTime<Utc>,

  pub likes: i32,

  pub content: String,
  pub tournament: bool,
}

pub async fn new_projects(db_pool: PgPool) -> WebResult<impl Reply> {
  match db::get_newest_projects(&db_pool).await {
    Ok(projects) => {
      return Ok(json(&projects));
    }
    Err(err) => {
      warn!("Error getting newest projects: {err}");
      return Err(reject::custom(error::Error::ServerProblem));
    }
  }
}

// GET v1/projects/USER_NAME/PROJECT_NAME
pub async fn get_project(username: String, project_name: String, user_uid: Option<Uuid>, db_pool: PgPool) -> WebResult<impl Reply> {
  let can_see_private = is_authorized(&user_uid, &username, &db_pool).await;

  match db::get_project_by_ownername_projectname(&username, &project_name, can_see_private, &db_pool).await {
    Ok(response) => {
      if let Some(project) = response {
        let response = FullProjectResponse {
          id: project.id,
          name: project.name,
          display_name: project.display_name,
          owner_id: project.owner_id,
          owner_name: username.clone(),
          private: project.private,
          description: project.description,
          content: project.content,
          github_url: project.github_url,
          website_url: project.website_url,
          likes: project.likes,
          created_at: project.created_at,
          updated_at: project.updated_at,
          tournament: project.tournament,
          url: format!("https://ckziucodefest.pl/p/{}/{}", &username, &project_name),
        };
        return Ok(json(&response));
      }
    }
    Err(err) => {
      warn!("Detected server problem @ DB PROJECT GET: {}", err);
      return Err(reject::custom(error::Error::ServerProblem));
    }
  }

  Err(reject::custom(error::Error::ProjectNotFound))
}

#[derive(Deserialize)]
pub struct PatchProject {
  pub display_name: Option<String>,
  pub private: Option<bool>,
  pub description: Option<String>,
  pub content: Option<String>,
  pub github_url: Option<String>,
  pub website_url: Option<String>,
  pub tournament: Option<bool>,
}

// DELETE v1/projects/USER_NAME/PROJECT_NAME
pub async fn delete_project(username: String, project_name: String, user_uid: Option<Uuid>, db_pool: PgPool) -> WebResult<impl Reply> {
  let authorized = is_authorized(&user_uid, &username, &db_pool).await;
  // Reject unauthorized
  if !authorized {
    return Err(reject::custom(error::Error::Unauthorized));
  }
  let user_id = user_uid.unwrap();

  // Remove from database
  match db::delete_project(&user_id, &project_name, &db_pool).await {
    Ok(_) => (),
    Err(err) => {
      warn!("Failed to delete project: {err}");
      return Err(reject::custom(error::Error::ServerProblem));
    }
  }

  Ok(warp::reply::with_status("OK", warp::http::StatusCode::ACCEPTED))
}

// PATCH v1/projects/USER_NAME/PROJECT_NAME
pub async fn patch_project(username: String, project_name: String, user_uid: Option<Uuid>, mut patch: PatchProject, db_pool: PgPool) -> WebResult<impl Reply> {
  let authorized = is_authorized(&user_uid, &username, &db_pool).await;
  if !authorized {
    return Err(reject::custom(error::Error::Unauthorized));
  }
  if patch.display_name.is_none() && patch.private.is_none() && patch.description.is_none() && patch.content.is_none() && patch.github_url.is_none() && patch.website_url.is_none() && patch.tournament.is_none() {
    return Ok(json(&PostProjectResponse {
      success: true,
      created: false,
      message: "Brak danych".into(),
    }));
  }

  if patch.display_name.is_some() {
    patch.display_name = match validate_display_name(patch.display_name.unwrap()) {
      Ok(display_name) => Some(display_name),
      Err(message) => {
        return Ok(json(&PostProjectResponse {
          success: false,
          created: false,
          message,
        }));
      }
    };
  }

  if patch.description.is_some() {
    patch.description = match validate_description(patch.description) {
      Ok(description) => description,
      Err(message) => {
        return Ok(json(&PostProjectResponse {
          success: false,
          created: false,
          message,
        }));
      }
    };
  }
  let user_uid = user_uid.unwrap();
  match db::patch_project(&user_uid, patch, &project_name, &db_pool).await {
    Ok(_) => {}
    Err(err) => {
      warn!("{err}");
      return Err(reject::custom(error::Error::ServerProblem));
    }
  };


  Ok(json(&PostProjectResponse {
    success: true,
    created: true,
    message: "Zaktualizowano dane".into(),
  }))
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
    private: body.private,
    description: project_description,
    content: String::new(),
    github_url: None,
    website_url: None,
    likes: 0,
    tournament: false,
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