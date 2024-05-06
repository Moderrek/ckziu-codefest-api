use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::user::models::User;

#[derive(Deserialize)]
pub struct PostProjectBody {
  pub name: String,
  pub display_name: String,
  pub description: Option<String>,
  pub private: bool,
}

#[derive(Serialize)]
pub struct PostProjectResponse {
  pub success: bool,
  pub message: String,
  pub created: bool,
}

#[derive(Serialize)]
pub struct GetProjectResponse {
  pub id: Uuid,
  pub name: String,
  pub display_name: String,

  pub owner: User,

  pub private: bool,
  pub description: Option<String>,

  pub likes: u32,

  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}
