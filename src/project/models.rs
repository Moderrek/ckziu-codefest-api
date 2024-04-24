use chrono::{DateTime, Utc};
use chrono::serde::ts_milliseconds;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(FromRow, Serialize, Deserialize)]
pub struct Project {
  pub id: Uuid,
  pub name: String,
  pub display_name: String,

  pub owner_id: Uuid,

  pub private: bool,
  pub description: Option<String>,

  pub likes: i32,

  #[serde(with = "ts_milliseconds")]
  pub created_at: DateTime<Utc>,
  #[serde(with = "ts_milliseconds")]
  pub updated_at: DateTime<Utc>,
}
