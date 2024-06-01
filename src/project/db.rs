use sqlx::PgPool;
use uuid::Uuid;

use crate::project::api::PatchProject;
use crate::project::models::Project;

use super::api::FullProjectResponse;
use super::models::{ContestProject, ProjectCard};

pub async fn delete_project(owner_id: &Uuid, project_name: &String, pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
  let query = r"DELETE FROM projects USING users WHERE projects.owner_id = $1 AND projects.name = $2";

  let mut transaction = pool.begin().await?;

  sqlx::query(query)
    .bind(owner_id)
    .bind(project_name)
    .execute(&mut *transaction)
    .await?;

  transaction.commit().await?;

  Ok(())
}

#[allow(dead_code)]
pub async fn get_projects_by_ownerid(
  owner_id: &Uuid,
  pool: &PgPool,
) -> Result<Vec<Project>, Box<dyn std::error::Error>> {
  let query = r"SELECT * FROM projects WHERE owner_id = $1 ORDER BY projects.updated_at DESC";

  let result: Vec<Project> = sqlx::query_as(query).bind(owner_id).fetch_all(pool).await?;

  Ok(result)
}

#[allow(dead_code)]
pub async fn get_projects_by_ownername(
  username: &String,
  pool: &PgPool,
) -> Result<Vec<Project>, Box<dyn std::error::Error>> {
  let query = r"SELECT content, website_url, github_url, tournament, projects.id, projects.name, projects.display_name, projects.owner_id, projects.private, projects.description, projects.likes, projects.created_at, projects.updated_at, users.id AS userid, users.name AS username FROM projects INNER JOIN users ON projects.owner_id = users.id WHERE users.name = $1 ORDER BY projects.updated_at DESC";

  let result: Vec<Project> = sqlx::query_as(query).bind(username).fetch_all(pool).await?;

  Ok(result)
}

pub async fn get_newest_projects(pool: &PgPool) -> Result<Vec<FullProjectResponse>, Box<dyn std::error::Error>> {
  let query = r"SELECT content,
  projects.website_url,
  projects.github_url,
  projects.tournament,
  projects.id,
  projects.name,
  projects.display_name,
  projects.owner_id,
  projects.private,
  projects.description,
  projects.likes,
  projects.created_at,
  projects.updated_at,
  users.id                                                            AS userid,
  users.name                                                          AS owner_name,
  concat('https://ckziucodefest.pl/p/', users.name, '/', projects.name) as url
FROM projects
    INNER JOIN users ON projects.owner_id = users.id
WHERE projects.private = false
ORDER BY updated_at DESC
LIMIT 6";
  let result: Vec<FullProjectResponse> = sqlx::query_as(query).fetch_all(pool).await?;

  Ok(result)
}


pub async fn get_contest_projects(pool: &PgPool) -> Result<Vec<ContestProject>, Box<dyn std::error::Error>> {
  let query = r#"
  SELECT
    projects.id,
    projects.name,
    projects.display_name,


    projects.owner_id as "owner_id",
    users.name as "owner_name",
    users.display_name as "owner_display_name",
    
    projects.description,

    projects.votes,
    projects.created_at,
    projects.updated_at

  FROM projects
  INNER JOIN users ON projects.owner_id = users.id
  WHERE projects.tournament = true AND private = false
  ORDER BY projects.created_at DESC
  "#;

  let result: Vec<ContestProject> = sqlx::query_as(query).fetch_all(pool).await?;

  Ok(result)
}

pub async fn get_project_by_ownername_projectname(
  username: &String,
  project_name: &String,
  can_be_private: bool,
  pool: &PgPool,
) -> Result<Option<Project>, Box<dyn std::error::Error>> {
  let query = r"SELECT tournament, projects.id, owner_id, private, projects.name, projects.display_name, github_url, website_url, content, description, likes, projects.created_at, projects.updated_at, users.id AS userid, users.name AS username FROM projects INNER JOIN users ON projects.owner_id = users.id WHERE users.name = $1 AND projects.name = $2 AND (projects.private = false OR projects.private = $3) LIMIT 1";

  let result: Option<Project> = sqlx::query_as(query)
    .bind(username)
    .bind(project_name)
    .bind(can_be_private)
    .fetch_optional(pool)
    .await?;

  Ok(result)
}

const HAS_PROJECT_BY_ID_QUERY: &str =
  r"SELECT id FROM projects WHERE owner_id = $1 AND name = $2 LIMIT 1";

pub async fn has_project_by_id(
  owner_id: &Uuid,
  project_name: &String,
  pool: &PgPool,
) -> Result<bool, Box<dyn std::error::Error>> {
  let result: Option<(Uuid, )> = sqlx::query_as(HAS_PROJECT_BY_ID_QUERY)
    .bind(owner_id)
    .bind(project_name)
    .fetch_optional(pool)
    .await?;
  Ok(result.is_some())
}

pub async fn create_project(
  project: &Project,
  pool: &PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
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

pub async fn patch_project(owner_id: &Uuid, patch: PatchProject, projectname: &String, pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
  let mut transaction = pool.begin().await?;

  sqlx::query(r"UPDATE projects SET updated_at = now() WHERE name = $1")
    .bind(projectname)
    .execute(&mut *transaction)
    .await?;

  if let Some(display_name) = patch.display_name {
    let query = r"UPDATE projects SET display_name = $1 WHERE name = $2";
    sqlx::query(query)
      .bind(display_name)
      .bind(projectname)
      .execute(&mut *transaction)
      .await?;
  }
  if let Some(description) = patch.description {
    let query = r"UPDATE projects SET description = $1 WHERE name = $2";
    sqlx::query(query)
      .bind(description)
      .bind(projectname)
      .execute(&mut *transaction)
      .await?;
  }
  if let Some(website_url) = patch.website_url {
    let query = r"UPDATE projects SET website_url = $1 WHERE name = $2";
    sqlx::query(query)
      .bind(website_url)
      .bind(projectname)
      .execute(&mut *transaction)
      .await?;
  }
  if let Some(content) = patch.content {
    let query = r"UPDATE projects SET content = $1 WHERE name = $2";
    sqlx::query(query)
      .bind(content)
      .bind(projectname)
      .execute(&mut *transaction)
      .await?;
  }
  if let Some(github_url) = patch.github_url {
    let query = r"UPDATE projects SET github_url = $1 WHERE name = $2";
    sqlx::query(query)
      .bind(github_url)
      .bind(projectname)
      .execute(&mut *transaction)
      .await?;
  }
  if let Some(private) = patch.private {
    let query = r"UPDATE projects SET private = $1 WHERE name = $2";
    sqlx::query(query)
      .bind(private)
      .bind(projectname)
      .execute(&mut *transaction)
      .await?;
    if private {
      let query = r"UPDATE projects SET tournament = false WHERE name = $1";
      sqlx::query(query)
        .bind(projectname)
        .execute(&mut *transaction)
        .await?;
    }
  }
  if let Some(tournament) = patch.tournament {
    let query = r"UPDATE projects SET tournament = false WHERE owner_id = $1";
    sqlx::query(query)
      .bind(owner_id)
      .execute(&mut *transaction)
      .await?;
    if tournament {
      let query = r"UPDATE projects SET tournament = true WHERE name = $1";
      sqlx::query(query)
        .bind(projectname)
        .execute(&mut *transaction)
        .await?;
    }
  }


  transaction.commit().await?;

  Ok(())
}