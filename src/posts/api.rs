use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
use reply::json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Decode, FromRow, PgPool};
use tracing::warn;
use uuid::Uuid;
use warp::{reply, Reply};

use crate::error::Error;
use crate::prelude::WebResult;

#[derive(Deserialize)]
pub struct CreatePost {
    pub content: String,
}

#[derive(Deserialize, Serialize, FromRow)]
pub struct Post {
    pub id: i32,
    pub owner_id: Uuid,
    pub content: String,
    #[serde(with = "ts_milliseconds")]
    pub created_at: DateTime<Utc>,
    pub likes: i32,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct PostWithOwner {
    pub id: i32,
    pub owner: PostOwner,
    pub content: String,
    #[serde(with = "ts_milliseconds")]
    pub created_at: DateTime<Utc>,
    pub likes: i32,
}

#[derive(Serialize, Deserialize, FromRow, Decode)]
pub struct PostOwner {
    pub id: Uuid,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub name: String,
    pub flags: i32,
}

pub async fn get_posts(db_pool: PgPool) -> WebResult<impl Reply> {
    let posts: Vec<(i32, String, DateTime<Utc>, i32, Uuid, String, String, i32)> = sqlx::query_as(
    r#"
    SELECT posts.id, posts.content, posts.created_at, posts.likes, users.id as "owner.id", users.name as "owner.name", users.display_name as "owner.display_name", users.flags as "owner.flags"
    FROM posts
    JOIN users ON posts.owner_id = users.id
    ORDER BY posts.created_at DESC
    "#,
  )
    .fetch_all(&db_pool)
    .await
    .map_err(|_| Error::ServerProblem)?;

    let posts = posts
        .into_iter()
        .map(
            |(
                id,
                content,
                created_at,
                likes,
                owner_id,
                owner_name,
                owner_display_name,
                owner_flags,
            )| PostWithOwner {
                id,
                content,
                owner: PostOwner {
                    id: owner_id,
                    name: owner_name,
                    display_name: owner_display_name,
                    flags: owner_flags,
                },
                created_at,
                likes,
            },
        )
        .collect::<Vec<_>>();

    Ok(json(&posts))
}

pub async fn create_post(
    auth: Option<Uuid>,
    post: CreatePost,
    db_pool: PgPool,
) -> WebResult<impl Reply> {
    // Reject if user is not authenticated
    let user_id = auth.ok_or(Error::Unauthorized)?;

    // Perform insert
    sqlx::query(
        r#"
    INSERT INTO posts (owner_id, content)
    VALUES ($1, $2)
    "#,
    )
    .bind(&user_id)
    .bind(&post.content)
    .execute(&db_pool)
    .await
    .map_err(|err| {
        warn!("Failed to create post: {err}");
        Error::ServerProblem
    })?;

    // Operation success
    Ok(reply::with_status(
        reply::json(&json!({ "success": true })),
        warp::http::StatusCode::CREATED,
    ))
}

pub async fn set_like_post(
    user_uid: Uuid,
    post_uid: i32,
    db_pool: PgPool,
    liked: bool,
) -> WebResult<impl Reply> {
    // Check is post exists
    let post_exists: bool = sqlx::query_scalar(
        r#"
    SELECT EXISTS(SELECT 1 FROM posts WHERE id = $1)
    "#,
    )
    .bind(&post_uid)
    .fetch_one(&db_pool)
    .await
    .map_err(|err| {
        warn!("Failed to check is post exists: {err}");
        Error::ServerProblem
    })?;

    // Reject if post does not exist
    if !post_exists {
        return Ok(reply::with_status(
            reply::json(&json!({ "success": false, "message": "Post does not exist" })),
            warp::http::StatusCode::NOT_FOUND,
        ));
    }

    // Check if user already liked the post
    let already_liked: bool = sqlx::query_scalar(
        r#"
    SELECT EXISTS(SELECT 1 FROM posts_likes WHERE post_id = $1 AND user_id = $2)
    "#,
    )
    .bind(&post_uid)
    .bind(&user_uid)
    .fetch_one(&db_pool)
    .await
    .map_err(|err| {
        warn!("Failed to check is post liked: {err}");
        Error::ServerProblem
    })?;

    // Insert or delete like
    if liked {
        if already_liked {
            return Ok(reply::with_status(
                reply::json(&json!({ "success": false, "message": "Already liked" })),
                warp::http::StatusCode::CONFLICT,
            ));
        }

        sqlx::query(
            r#"
      INSERT INTO posts_likes (post_id, user_id)
      VALUES ($1, $2)
      "#,
        )
        .bind(&post_uid)
        .bind(&user_uid)
        .execute(&db_pool)
        .await
        .map_err(|err| {
            warn!("Failed to like post: {err}");
            Error::ServerProblem
        })?;

        // Increment post likes counter
        sqlx::query(
            r#"
      UPDATE posts
      SET likes = likes + 1
      WHERE id = $1
      "#,
        )
        .bind(&post_uid)
        .execute(&db_pool)
        .await
        .map_err(|err| {
            warn!("Failed to increment post likes counter: {err}");
            Error::ServerProblem
        })?;
    } else {
        if !already_liked {
            return Ok(reply::with_status(
                reply::json(&json!({ "success": false, "message": "Not liked" })),
                warp::http::StatusCode::CONFLICT,
            ));
        }

        sqlx::query(
            r#"
      DELETE FROM posts_likes
      WHERE post_id = $1 AND user_id = $2
      "#,
        )
        .bind(&post_uid)
        .bind(&user_uid)
        .execute(&db_pool)
        .await
        .map_err(|err| {
            warn!("Failed to unlike post: {err}");
            Error::ServerProblem
        })?;

        // Decrement post likes counter
        sqlx::query(
            r#"
      UPDATE posts
      SET likes = likes - 1
      WHERE id = $1
      "#,
        )
        .bind(&post_uid)
        .execute(&db_pool)
        .await
        .map_err(|err| {
            warn!("Failed to decrement post likes counter: {err}");
            Error::ServerProblem
        })?;
    }
    // Operation success
    Ok(reply::with_status(
        reply::json(&json!({ "success": true })),
        warp::http::StatusCode::OK,
    ))
}

pub async fn like_post(post_id: i32, auth: Option<Uuid>, db_pool: PgPool) -> WebResult<impl Reply> {
    set_like_post(auth.ok_or(Error::Unauthorized)?, post_id, db_pool, true).await
}

pub async fn unlike_post(
    post_id: i32,
    auth: Option<Uuid>,
    db_pool: PgPool,
) -> WebResult<impl Reply> {
    set_like_post(auth.ok_or(Error::Unauthorized)?, post_id, db_pool, false).await
}
