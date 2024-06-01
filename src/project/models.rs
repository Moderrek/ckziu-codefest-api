use chrono::{DateTime, Utc};
use chrono::serde::ts_milliseconds;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::Decode;
use uuid::Uuid;

#[derive(FromRow, Serialize, Deserialize)]
pub struct Project {
  pub id: Uuid,
  pub name: String,
  pub display_name: String,

  pub owner_id: Uuid,

  pub private: bool,
  pub description: Option<String>,

  pub content: String,
  pub github_url: Option<String>,
  pub website_url: Option<String>,

  pub tournament: bool,
  pub likes: i32,

  #[serde(with = "ts_milliseconds")]
  pub created_at: DateTime<Utc>,
  #[serde(with = "ts_milliseconds")]
  pub updated_at: DateTime<Utc>,
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct ProjectCard {
  pub id: Uuid,
  pub name: String,
  pub display_name: String,

  pub owner_id: Uuid,

  pub private: bool,
  pub description: Option<String>,

  pub tournament: bool,
  pub likes: i32,

  #[serde(with = "ts_milliseconds")]
  pub created_at: DateTime<Utc>,
  #[serde(with = "ts_milliseconds")]
  pub updated_at: DateTime<Utc>,
}

#[derive(FromRow, Serialize, Deserialize, Decode)]
pub struct ContestProjectOwner {

}

#[derive(FromRow, Serialize, Deserialize)]
pub struct ContestProject {
  pub id: Uuid,
  pub name: String,
  #[serde(rename = "displayName")]
  pub display_name: String,

  #[serde(rename = "ownerId")]
  pub owner_id: Uuid,
  #[serde(rename = "ownerName")]
  pub owner_name: String,
  #[serde(rename = "ownerDisplayName")]
  pub owner_display_name: String,
  
  pub description: Option<String>,
  
  pub votes: i32,

  #[serde(rename = "createdAt", with = "ts_milliseconds")]
  pub created_at: DateTime<Utc>,
  #[serde(rename = "updatedAt", with = "ts_milliseconds")]
  pub updated_at: DateTime<Utc>,
}


#[derive(FromRow, Serialize, Deserialize)]
pub struct ProjectCardWithOwner {
  pub id: Uuid,
  pub name: String,
  pub display_name: String,

  pub owner_id: Uuid,

  pub private: bool,
  pub description: Option<String>,

  pub tournament: bool,
  pub likes: i32,

  #[serde(with = "ts_milliseconds")]
  pub created_at: DateTime<Utc>,
  #[serde(with = "ts_milliseconds")]
  pub updated_at: DateTime<Utc>,
}
