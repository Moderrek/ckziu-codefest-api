use sqlx::PgPool;
use warp::Filter;

use crate::{auth::header::with_auth, db::with_db};

use super::api;

pub fn routes(db_pool: &PgPool) -> impl Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
  let post = warp::path!("projects")
    .and(warp::post())
    .and(warp::path::end())
    .and(with_auth())
    .and(warp::body::content_length_limit(1024 * 16))
    .and(warp::body::json())
    .and(with_db(db_pool.clone()))
    .and_then(api::create_project);

  let list = warp::path("projects")
    .and(warp::get())
    .and(warp::path::end())
    .and(with_db(db_pool.clone()))
    .and_then(api::new_projects);

  let get = warp::path!("projects" / String / String)
    .and(warp::get())
    .and(warp::path::end())
    .and(with_auth())
    .and(with_db(db_pool.clone()))
    .and_then(api::get_project);

  let patch = warp::path!("projects" / String / String)
    .and(warp::patch())
    .and(warp::path::end())
    .and(with_auth())
    .and(warp::body::content_length_limit(1024 * 16))
    .and(warp::body::json())
    .and(with_db(db_pool.clone()))
    .and_then(api::patch_project);

  let delete = warp::path!("projects" / String / String)
    .and(warp::delete())
    .and(warp::path::end())
    .and(with_auth())
    .and(with_db(db_pool.clone()))
    .and_then(api::delete_project);

  get
    .or(list)
    .or(post)
    .or(patch)
    .or(delete)
}
