use sqlx::PgPool;

use crate::project::models::Project;
use crate::user::models::User;
use crate::user::responses::ProfileResponse;

pub async fn create_user(user: &User, pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
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

pub async fn get_user(name: &String, pool: &PgPool) -> Result<Option<User>, Box<dyn std::error::Error>> {
  let query = "SELECT name, display_name, id, bio, created_at, updated_at, flags FROM users WHERE name = $1 LIMIT 1";

  let mut transaction = pool.begin().await?;

  let result: Option<User> = sqlx::query_as(query)
    .bind(name)
    .fetch_optional(&mut *transaction)
    .await?;

  transaction.commit().await?;

  Ok(result)
}

pub async fn get_profile(username: &String, pool: &PgPool) -> Result<Option<ProfileResponse>, Box<dyn std::error::Error>> {
  let query = "SELECT * FROM users WHERE name = $1 LIMIT 1";
  let query_projects = "SELECT * FROM projects INNER JOIN users ON projects.owner_id = users.id WHERE users.name = $1 ORDER BY projects.updated_at";

  let mut transaction = pool.begin().await?;

  let result: Option<User> = sqlx::query_as(query)
    .bind(username)
    .fetch_optional(&mut *transaction)
    .await?;

  let projects: Vec<Project> = sqlx::query_as(query_projects)
    .bind(username)
    .fetch_all(&mut *transaction)
    .await?;

  transaction.commit().await?;

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

pub async fn update_bio(name: &String, pool: &PgPool, bio: &String) -> Result<(), Box<dyn std::error::Error>> {
  let query = "UPDATE users SET bio = $1 WHERE name = $2";

  let mut transaction = pool.begin().await?;

  sqlx::query(query)
    .bind(name)
    .bind(bio)
    .execute(&mut *transaction)
    .await?;

  Ok(())
}
