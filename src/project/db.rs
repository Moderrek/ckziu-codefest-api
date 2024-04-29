use sqlx::PgPool;
use uuid::Uuid;

use crate::project::models::Project;

pub async fn get_projects_by_ownerid(owner_id: &Uuid, pool: &PgPool) -> Result<Vec<Project>, Box<dyn std::error::Error>> {
  let query = r"SELECT * FROM projects WHERE owner_id = $1";

  let result: Vec<Project> = sqlx::query_as(query)
    .bind(owner_id)
    .fetch_all(pool)
    .await?;

  Ok(result)
}

pub async fn get_projects_by_ownername(username: &String, pool: &PgPool) -> Result<Vec<Project>, Box<dyn std::error::Error>> {
  let query = r"SELECT * FROM projects INNER JOIN users ON projects.owner_id = users.id WHERE users.name = $1 ORDER BY projects.updated_at";

  let result: Vec<Project> = sqlx::query_as(query)
    .bind(username)
    .fetch_all(pool)
    .await?;

  Ok(result)
}

pub async fn get_project_by_ownername_projectname(username: &String, project_name: &String, pool: &PgPool) -> Result<Option<Project>, Box<dyn std::error::Error>> {
  let query = r"SELECT * FROM projects INNER JOIN users ON projects.owner_id = users.id WHERE users.name = $1 AND projects.name = $2 ORDER BY projects.updated_at LIMIT 1";

  let result: Option<Project> = sqlx::query_as(query)
    .bind(username)
    .bind(project_name)
    .fetch_optional(pool)
    .await?;

  Ok(result)
}


pub async fn create_project(project: &Project, pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
  let query = r"INSERT INTO projects (id, name, display_name, owner_id, private, description, likes, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)";

  let mut transaction = pool.begin().await?;

  sqlx::query(query)
    .bind(project.id)
    .bind(&project.name)
    .bind(&project.display_name)
    .bind(project.owner_id)
    .bind(project.private)
    .bind(&project.description)
    .bind(project.likes)
    .bind(project.created_at)
    .bind(project.updated_at)
    .execute(&mut *transaction)
    .await?;

  transaction.commit().await?;

  Ok(())
}