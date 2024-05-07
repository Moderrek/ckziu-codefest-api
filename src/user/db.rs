use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::project::models::Project;
use crate::user::models::User;
use crate::user::responses::ProfileResponse;

const GET_USER_QUERY: &str = r"SELECT name, display_name, id, bio, created_at, updated_at, flags FROM users WHERE name = $1 LIMIT 1";
const GET_USER_BY_ID_QUERY: &str = r"SELECT * FROM users WHERE id = $1 LIMIT 1";

pub async fn get_user(
  name: &String,
  pool: &PgPool,
) -> Result<Option<User>, Box<dyn std::error::Error>> {
  let result: Option<User> = sqlx::query_as(GET_USER_QUERY)
    .bind(name)
    .fetch_optional(pool)
    .await?;

  Ok(result)
}

pub async fn get_user_by_id(
  id: &Uuid,
  pool: &PgPool,
) -> Result<Option<User>, Box<dyn std::error::Error>> {
  let result: Option<User> = sqlx::query_as(GET_USER_BY_ID_QUERY)
    .bind(id)
    .fetch_optional(pool)
    .await?;

  Ok(result)
}

pub async fn get_profile(
  username: &String,
  pool: &PgPool,
) -> Result<Option<ProfileResponse>, Box<dyn std::error::Error>> {
  let query = "SELECT * FROM users WHERE name = $1 LIMIT 1";
  let query_projects = "SELECT  projects.id, projects.name, projects.display_name, projects.owner_id, projects.private, projects.description, projects.likes, projects.created_at, projects.updated_at, users.id AS userid, users.name AS username  FROM projects INNER JOIN users ON projects.owner_id = users.id WHERE users.name = $1 ORDER BY projects.updated_at";

  let result: Option<User> = sqlx::query_as(query)
    .bind(username)
    .fetch_optional(pool)
    .await?;

  let projects: Vec<Project> = sqlx::query_as(query_projects)
    .bind(username)
    .fetch_all(pool)
    .await?;

  if result.is_none() {
    return Ok(None);
  }
  let user = result.unwrap();

  let response = ProfileResponse {
    name: user.name,
    display_name: user.display_name,
    id: user.id,
    bio: user.bio,
    projects,
    created_at: user.created_at,
    updated_at: user.updated_at,
    flags: user.flags,
  };

  Ok(Some(response))
}

pub async fn update_bio(
  uid: &Uuid,
  pool: &PgPool,
  bio: &String,
) -> Result<(), Box<dyn std::error::Error>> {
  let query = "UPDATE users SET bio = $1, updated_at = $2 WHERE id = $3";

  let mut transaction = pool.begin().await?;

  sqlx::query(query)
    .bind(bio)
    .bind(Utc::now())
    .bind(uid)
    .execute(&mut *transaction)
    .await?;

  transaction.commit().await?;

  Ok(())
}

pub async fn update_display_name(
  uid: &Uuid,
  pool: &PgPool,
  display_name: &String,
) -> Result<(), Box<dyn std::error::Error>> {
  let query = "UPDATE users SET display_name = $1, updated_at = $2 WHERE id = $3";

  let mut transaction = pool.begin().await?;

  sqlx::query(query)
    .bind(display_name)
    .bind(Utc::now())
    .bind(uid)
    .execute(&mut *transaction)
    .await?;

  transaction.commit().await?;

  Ok(())
}

pub async fn get_info(uuid: &Uuid, pool: &PgPool) -> Result<(String, ), Box<dyn std::error::Error>> {
  let query = "SELECT name FROM users WHERE id = $1 LIMIT 1";

  let result: (String, ) = sqlx::query_as(query).bind(uuid).fetch_one(pool).await?;

  Ok(result)
}

const GET_USERCOUNT_QUERY: &str = r"SELECT COUNT(*) FROM users";

pub async fn get_userscount(pool: &PgPool) -> Result<i16, Box<dyn std::error::Error>> {
  let result: (i16, ) = sqlx::query_as(GET_USERCOUNT_QUERY).fetch_one(pool).await?;

  Ok(result.0)
}
