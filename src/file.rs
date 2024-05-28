use sqlx::PgPool;
use tracing::warn;
use warp::{reject, Reply};

use db::get_user_avatar_url;
use error::Error;

use crate::{error, WebResult};
use crate::user::db;

pub async fn get_profile(username: String, db_pool: PgPool) -> WebResult<impl Reply> {
  let avatar: String;
  match get_user_avatar_url(&username, &db_pool).await {
    Ok(data) => {
      if data.is_none() {
        return Err(reject::custom(Error::NotFound));
      }
      let data = data.unwrap();
      if data.0.is_none() {
        return Err(reject::custom(Error::CannotFindFile));
      }
      avatar = data.0.unwrap();
    }
    Err(err) => {
      warn!("{err}");
      return Err(reject::custom(Error::ServerProblem));
    }
  };

  let filepath = format!("uploads/{}", avatar);
  let bytes = tokio::fs::read(filepath).await.unwrap();
  Ok(warp::reply::with_header(bytes, "content-type", "image/png"))
}