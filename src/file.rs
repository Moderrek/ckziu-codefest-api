use sqlx::PgPool;
use tracing::warn;
use warp::{reject, Reply};

use db::get_user_avatar_url;
use error::Error;

use crate::{error, WebResult};
use crate::user::db;

pub async fn get_profile(username: String, db_pool: PgPool) -> WebResult<impl Reply> {
  let avatar = match get_user_avatar_url(&username, &db_pool).await {
    Ok(data) => {
      if data.is_none() {
        // Reject because the user does not exist
        return Err(reject::custom(Error::NotFound));
      }
      let data = data.unwrap();
      if data.0.is_none() {
        // Reject because the user has no avatar
        return Err(reject::custom(Error::CannotFindFile));
      }
      data.0.unwrap()
    }
    Err(err) => {
      // Encoutered an database error while fetching
      warn!("Failed to fetch user avatar url: {err}");
      return Err(reject::custom(Error::ServerProblem));
    }
  };
  
  let filepath = format!("uploads/{}", avatar);
  // Asynchronously read the file from the disk
  let bytes = tokio::fs::read(filepath).await.unwrap();

  // Return the file as a response
  Ok(warp::reply::with_header(bytes, "content-type", "image/png"))
}