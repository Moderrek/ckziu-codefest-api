use sqlx::{Error, PgPool, Postgres};
use warp::reply::{json, Reply};

use crate::WebResult;

pub async fn create_pool() -> Result<PgPool, Error> {
  let pool = sqlx::postgres::PgPoolOptions::new()
    .max_connections(5)
    .connect(dotenv!("DATABASE_URL"))
    .await?;

  sqlx::migrate!("./migrations").run(&pool).await?;

  Ok(pool)
}

pub async fn database_handler(pool: sqlx::Pool<Postgres>) -> WebResult<impl Reply> {
  let row: (i64, ) = sqlx::query_as("SELECT $1")
    .bind(150_i64)
    .fetch_one(&pool)
    .await
    .unwrap();

  Ok(json(&row))
}
