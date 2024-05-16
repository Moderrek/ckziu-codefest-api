use sqlx::PgPool;
use warp::Filter;

use crate::{auth::header::with_auth, database::with_db};

use super::api;

pub fn routes(db_pool: &PgPool) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
  let list = warp::path!("users")
    .and(warp::path::end())
    .and(warp::query())
    .and(with_db(db_pool.clone()))
    .and_then(api::list_users);

  let get = warp::path!("users" / String)
    .and(warp::get())
    .and(warp::path::end())
    .and(with_db(db_pool.clone()))
    .and_then(api::get_user);

  let patch = warp::path!("users" / String)
    .and(warp::patch())
    .and(warp::path::end())
    .and(warp::body::json())
    .and(with_auth())
    .and(with_db(db_pool.clone()))
    .and_then(api::patch_user);

  list
    .or(get)
    .or(patch)
}
