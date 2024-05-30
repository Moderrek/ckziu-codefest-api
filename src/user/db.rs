use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::posts::api::PostWithLiked;
use crate::project::models::ProjectCard;
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

pub async fn get_users(
    page_size: u32,
    page: u32,
    pool: &PgPool,
) -> Result<Vec<User>, Box<dyn std::error::Error>> {
    let users: Vec<User> =
        sqlx::query_as(r"SELECT * FROM users ORDER BY updated_at DESC LIMIT $1 OFFSET $2")
            .bind(page_size as i32)
            .bind(page as i32 * page_size as i32)
            .fetch_all(pool)
            .await?;
    Ok(users)
}

pub async fn is_user_exists(
    uuid: &Uuid,
    pool: &PgPool,
) -> Result<bool, Box<dyn std::error::Error>> {
    let username = get_username(uuid, pool).await?;
    Ok(username.is_some())
}

pub async fn get_username(
    uuid: &Uuid,
    pool: &PgPool,
) -> Result<Option<(String,)>, Box<dyn std::error::Error>> {
    let result: Option<(String,)> = sqlx::query_as(r"SELECT name FROM users WHERE id = $1 LIMIT 1")
        .bind(uuid)
        .fetch_optional(pool)
        .await?;
    Ok(result)
}

pub async fn get_user_avatar_url(
    name: &String,
    pool: &PgPool,
) -> Result<Option<(Option<String>,)>, Box<dyn std::error::Error>> {
    let result: Option<(Option<String>,)> =
        sqlx::query_as(r"SELECT avatar FROM users WHERE name = $1 LIMIT 1")
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
    receiver: Option<Uuid>,
    username: &String,
    is_authorized: bool,
    pool: &PgPool,
) -> Result<Option<ProfileResponse>, Box<dyn std::error::Error>> {
    let query = "SELECT * FROM users WHERE name = $1 LIMIT 1";
    let query_projects = "SELECT tournament, projects.id, projects.name, projects.display_name, projects.owner_id, projects.private, projects.description, projects.likes, projects.created_at, projects.updated_at, users.id AS userid, users.name AS username  FROM projects INNER JOIN users ON projects.owner_id = users.id WHERE users.name = $1 AND (projects.private = false OR projects.private = $2) ORDER BY projects.updated_at DESC";
    let query_posts = r#"
    SELECT
      posts.id, 
      posts.owner_id,
      posts.content,
      posts.created_at,
      posts.likes,
      EXISTS(SELECT 1 FROM posts_likes WHERE post_id = posts.id AND $2 IS NOT NULL AND user_id = $2) as "is_liked_by_user"
    FROM posts
    INNER JOIN users ON posts.owner_id = users.id
    WHERE users.name = $1
    ORDER BY posts.updated_at DESC"#;

    let result: Option<User> = sqlx::query_as(query)
        .bind(username)
        .fetch_optional(pool)
        .await?;

    let projects: Vec<ProjectCard> = sqlx::query_as(query_projects)
        .bind(username)
        .bind(is_authorized)
        .fetch_all(pool)
        .await?;

    let posts: Vec<PostWithLiked> = sqlx::query_as(query_posts)
        .bind(username)
        .bind(receiver)
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
        posts,
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

pub async fn get_info(uuid: &Uuid, pool: &PgPool) -> Result<(String,), Box<dyn std::error::Error>> {
    let query = "SELECT name FROM users WHERE id = $1 LIMIT 1";

    let result: (String,) = sqlx::query_as(query).bind(uuid).fetch_one(pool).await?;

    Ok(result)
}

const GET_USERCOUNT_QUERY: &str = r"SELECT COUNT(*) FROM users";

pub async fn get_userscount(pool: &PgPool) -> Result<i16, Box<dyn std::error::Error>> {
    let result: (i16,) = sqlx::query_as(GET_USERCOUNT_QUERY).fetch_one(pool).await?;

    Ok(result.0)
}
