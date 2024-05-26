use std::convert::Infallible;

use sqlx::{Error, PgPool};
use warp::Filter;

pub async fn create_pool() -> Result<PgPool, Error> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(dotenv!("DATABASE_URL"))
        .await?;

    // sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

pub fn with_db(pool: PgPool) -> impl Filter<Extract = (PgPool,), Error = Infallible> + Clone {
    warp::any().map(move || pool.clone())
}
