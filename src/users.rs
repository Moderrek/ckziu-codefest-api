use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use chrono::serde::ts_milliseconds;
use log::info;
use sqlx::{FromRow, PgPool, Pool, Postgres};
use sqlx::postgres::PgSeverity::Error;
use uuid::Uuid;
use warp::reject;
use crate::{error, WebResult};
use warp::reply::{json, Reply};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CodeFestUser {
  pub name: String,
  pub display_name: String,
  pub id: Uuid,

  pub bio: Option<String>,

  #[serde(with = "ts_milliseconds")]
  pub created_at: DateTime<Utc>,
  #[serde(with = "ts_milliseconds")]
  pub updated_at: DateTime<Utc>,

  pub flags: i32,
}

impl CodeFestUser {

  pub fn is_flag_set(&self, index: u32) -> bool {
    1 == (self.flags & 1 << index)
  }

  pub fn set_flag(&mut self, index: u32, turn_on: bool) {
    let flag = 1 << index;
    if turn_on {
      // switch on a flag
      self.flags |= flag;
    } else {
      // switch off a flag
      self.flags &= !flag;
    }
  }

  pub fn is_staff(&self) -> bool {
    self.is_flag_set(0)
  }

  pub fn set_staff(&mut self, turn_on: bool) {
    self.set_flag(0, turn_on)
  }

  pub fn is_developer(&self) -> bool {
    self.is_flag_set(1)
  }

  pub fn set_developer(&mut self, turn_on: bool) {
    self.set_flag(1, turn_on)
  }

  pub fn is_teacher(&self) -> bool {
    self.is_flag_set(2)
  }

  pub fn set_teacher(&mut self, turn_on: bool) {
    self.set_flag(2, turn_on)
  }

}

pub async fn db_insert_user(user: &CodeFestUser, pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
  let mut transaction = pool.begin().await?;

  let query = "INSERT INTO users (name, display_name, id, bio, created_at, updated_at, flags) VALUES ($1, $2, $3, $4, $5, $6, $7)";

  sqlx::query(query)
    .bind(&user.name)
    .bind(&user.display_name)
    .bind(user.id)
    .bind(&user.bio)
    .bind(user.created_at)
    .bind(user.updated_at)
    .bind(user.flags)
    .execute(&mut *transaction)
    .await?;

  transaction.commit().await?;

  Ok(())
}

pub async fn db_get_user(name: &String, pool: &PgPool) -> Result<Option<CodeFestUser>, Box<dyn std::error::Error>> {
  let query = "SELECT name, display_name, id, bio, created_at, updated_at, flags FROM users WHERE name = $1 LIMIT 1";

  let mut transaction = pool.begin().await?;

  let result: Option<CodeFestUser> = sqlx::query_as(query)
    .bind(name)
    .fetch_optional(&mut *transaction)
    .await?;

  transaction.commit().await?;

  Ok(result)
}

pub async fn api_get_user(name: String, db: PgPool) -> WebResult<impl Reply> {
  info!("Searching for user: {}", &name);
  if let Some(user) = db_get_user(&name, &db).await.unwrap() {
    info!("Found {}", &name);
    return Ok(json(&user));
  }
  info!("Cannot Found {}", &name);
  Err(reject::custom(error::Error::NotFound))
}