use sqlx::PgPool;
use tracing::warn;
use uuid::Uuid;

use crate::error::Error;

pub mod api;
pub mod routes;

pub async fn has_user_liked_post(post_id: i32, user_id: &Uuid, db_pool: &PgPool) -> Result<bool, Box<dyn std::error::Error>> {
    let like: (bool,) = sqlx::query_as(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM posts_likes
            WHERE post_id = $1 AND user_id = $2
        ) AS liked
        "#
    )
    .bind(post_id)
    .bind(user_id)
    .fetch_one(db_pool)
    .await
    .map_err(|err| {
        warn!("Failed to check if user has liked post: {err}");
        Error::ServerProblem
    })?;

    Ok(like.0)
}
