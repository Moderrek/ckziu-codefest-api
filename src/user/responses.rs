use chrono::{DateTime, Utc};
use chrono::serde::ts_milliseconds;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use models::Project;

use crate::project::models;

#[derive(Serialize)]
pub struct ProfileResponse {
  pub name: String,
  pub display_name: String,
  pub id: Uuid,

  pub bio: Option<String>,

  pub projects: Vec<Project>,

  #[serde(with = "ts_milliseconds")]
  pub created_at: DateTime<Utc>,
  #[serde(with = "ts_milliseconds")]
  pub updated_at: DateTime<Utc>,

  pub flags: i32,
}

#[derive(Deserialize)]
pub struct UpdateBioBody {
  pub bio: String,
}

#[derive(Deserialize)]
pub struct UpdateDisplayNameBody {
  pub displayname: String,
}

#[derive(Serialize)]
pub struct UpdateBioResponse {
  pub success: bool,
  pub message: String,
}