use sqlx::{Error, PgPool};

pub async fn create_pool() -> Result<PgPool, Error> {
  let pool = sqlx::postgres::PgPoolOptions::new()
    .max_connections(5)
    .connect(dotenv!("DATABASE_URL"))
    .await?;

  sqlx::migrate!("./migrations").run(&pool).await?;

  Ok(pool)
}