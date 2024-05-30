use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
use reply::json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Decode, FromRow, PgPool};
use tracing::{info, warn};
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
    #[serde(rename = "liked")]
    pub is_liked_by_user: bool,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct PostWithLiked {
    pub id: i32,
    pub content: String,
    #[serde(with = "ts_milliseconds")]
    pub created_at: DateTime<Utc>,
    pub likes: i32,
    #[serde(rename = "liked")]
    pub is_liked_by_user: bool,
}

#[derive(Serialize, Deserialize, FromRow, Decode)]
pub struct PostOwner {
    pub id: Uuid,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub name: String,
    pub flags: i32,
}

pub async fn get_posts(user_uid: Option<Uuid>, db_pool: PgPool) -> WebResult<impl Reply> {
    type PostTuple = (
        i32,
        String,
        DateTime<Utc>,
        i32,
        Uuid,
        String,
        String,
        i32,
        bool,
    );

    let posts: Vec<PostTuple> = sqlx::query_as(
            r#"
            SELECT
                posts.id,
                posts.content,
                posts.created_at,
                posts.likes, 

                users.id as "owner.id", 
                users.name as "owner.name",
                users.display_name as "owner.display_name",
                users.flags as "owner.flags",

                EXISTS(SELECT 1 FROM posts_likes WHERE post_id = posts.id AND $1 IS NOT NULL AND user_id = $1) as "liked"

            FROM posts
            JOIN users ON posts.owner_id = users.id
            
            ORDER BY posts.created_at DESC
            "#,
    )
    .bind(user_uid)
    .fetch_all(&db_pool)
    .await
    .map_err(|err| {
        warn!("Failed to fetch posts: {err}");
        Error::ServerProblem
    })?;

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
                liked,
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
                is_liked_by_user: liked,
            },
        )
        .collect::<Vec<_>>();

    Ok(json(&posts))
}

// Reject if user is not authenticated
pub async fn create_post(
    auth: Option<Uuid>,
    post: CreatePost,
    db_pool: PgPool,
) -> WebResult<impl Reply> {
    let user_id = auth.ok_or(Error::Unauthorized)?;

    let post = CreatePost {
        content: post.content.trim().to_string(),
    };

    // Reject post content if it is empty or too long (over 240 characters)
    if post.content.is_empty() || post.content.chars().count() > 240 {
        return Ok(reply::with_status(
            reply::json(&json!({ "success": false, "message": "Nieprawidłowa długość" })),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Perform insert
    let created_post: PostWithLiked = sqlx::query_as(
        r#"
    INSERT INTO posts (owner_id, content)
    VALUES ($1, $2)
    RETURNING *, false as "is_liked_by_user"
    "#,
    )
    .bind(user_id)
    .bind(&post.content)
    .fetch_one(&db_pool)
    .await
    .map_err(|err| {
        warn!("Failed to create post: {err}");
        Error::ServerProblem
    })?;

    info!("Post created: {} by {}", created_post.id, user_id);

    // Operation success, return the created post
    Ok(reply::with_status(
        reply::json(&created_post),
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
    .bind(post_uid)
    .fetch_one(&db_pool)
    .await
    .map_err(|err| {
        warn!("Failed to check is post exists: {err}");
        Error::ServerProblem
    })?;

    // Reject if post does not exist
    if !post_exists {
        return Ok(reply::with_status(
            reply::json(&json!({ "success": false, "message": "Wpis nie istnieje" })),
            warp::http::StatusCode::NOT_FOUND,
        ));
    }

    // Check if user already liked the post
    let already_liked: bool = sqlx::query_scalar(
        r#"
    SELECT EXISTS(SELECT 1 FROM posts_likes WHERE post_id = $1 AND user_id = $2)
    "#,
    )
    .bind(post_uid)
    .bind(user_uid)
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
                reply::json(&json!({ "success": false, "message": "Już polubiono" })),
                warp::http::StatusCode::CONFLICT,
            ));
        }

        sqlx::query(
            r#"
      INSERT INTO posts_likes (post_id, user_id)
      VALUES ($1, $2)
      "#,
        )
        .bind(post_uid)
        .bind(user_uid)
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
        .bind(post_uid)
        .execute(&db_pool)
        .await
        .map_err(|err| {
            warn!("Failed to increment post likes counter: {err}");
            Error::ServerProblem
        })?;
    } else {
        if !already_liked {
            return Ok(reply::with_status(
                reply::json(&json!({ "success": false, "message": "Nie polubiono" })),
                warp::http::StatusCode::CONFLICT,
            ));
        }

        sqlx::query(
            r#"
      DELETE FROM posts_likes
      WHERE post_id = $1 AND user_id = $2
      "#,
        )
        .bind(post_uid)
        .bind(user_uid)
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
        .bind(post_uid)
        .execute(&db_pool)
        .await
        .map_err(|err| {
            warn!("Failed to decrement post likes counter: {err}");
            Error::ServerProblem
        })?;
    }

    info!("Post {post_uid} like status to {liked} changed by {user_uid}");

    // Operation success
    Ok(reply::with_status(
        reply::json(&json!({ "success": true, "message": "Operacja zakończona pomyślnie"})),
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

pub async fn delete_post(post_id: i32, auth: Option<Uuid>, db_pool: PgPool) -> WebResult<impl Reply> {
    let user_id = auth.ok_or(Error::Unauthorized)?;

    // Check if post exists
    let post_exists: bool = sqlx::query_scalar(
        r#"
    SELECT EXISTS(SELECT 1 FROM posts WHERE id = $1 AND owner_id = $2)
    "#,
    )
    .bind(post_id)
    .bind(user_id)
    .fetch_one(&db_pool)
    .await
    .map_err(|err| {
        warn!("Failed to check is post exists: {err}");
        Error::ServerProblem
    })?;

    // Reject if post does not exist
    if !post_exists {
        return Ok(reply::with_status(
            reply::json(&json!({ "success": false, "message": "Wpis nie istnieje" })),
            warp::http::StatusCode::NOT_FOUND,
        ));
    }

    // Delete post
    sqlx::query(
        r#"
    DELETE FROM posts
    WHERE id = $1
    "#,
    )
    .bind(post_id)
    .execute(&db_pool)
    .await
    .map_err(|err| {
        warn!("Failed to delete post: {err}");
        Error::ServerProblem
    })?;

    info!("Post deleted: {post_id} by {user_id}");

    // Operation success
    Ok(reply::with_status(
        reply::json(&json!({ "success": true, "message": "Wpis został usunięty"})),
        warp::http::StatusCode::OK,
    ))
}