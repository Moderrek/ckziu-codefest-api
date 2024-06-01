use sqlx::PgPool;
use warp::Filter;

use crate::auth::header::with_auth;
use crate::db::with_db;
use crate::posts::api;

pub fn routes(
    db_pool: &PgPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let post = warp::path!("posts")
        .and(warp::post())
        .and(warp::path::end())
        .and(with_auth())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .and(with_db(db_pool.clone()))
        .and_then(api::create_post);

    let list = warp::path!("posts")
        .and(warp::get())
        .and(warp::path::end())
        .and(with_auth())
        .and(with_db(db_pool.clone()))
        .and_then(api::get_posts);

    let like = warp::path!("posts" / i32 / "like")
        .and(warp::get())
        .and(warp::path::end())
        .and(with_auth())
        .and(with_db(db_pool.clone()))
        .and_then(api::like_post);

    let unlike = warp::path!("posts" / i32 / "unlike")
        .and(warp::get())
        .and(warp::path::end())
        .and(with_auth())
        .and(with_db(db_pool.clone()))
        .and_then(api::unlike_post);

    let delete = warp::path!("posts" / i32)
        .and(warp::delete())
        .and(warp::path::end())
        .and(with_auth())
        .and(with_db(db_pool.clone()))
        .and_then(api::delete_post);

    //
    // let get = warp::path!("posts" / Uuid)
    //   .and(warp::get())
    //   .and(warp::path::end())
    //   .and(with_auth())
    //   .and(with_db(db_pool.clone()))
    //   .and_then(api::get_post);
    //
    // let delete = warp::path!("posts" / Uuid)
    //   .and(warp::delete())
    //   .and(warp::path::end())
    //   .and(with_auth())
    //   .and(with_db(db_pool.clone()))
    //   .and_then(api::delete_post);

    // get
    //   .or(list)
    //   .or(post)
    // .or(delete)
    list.or(post).or(like).or(unlike).or(delete)
}
