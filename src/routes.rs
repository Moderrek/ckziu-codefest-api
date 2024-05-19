use std::sync::Arc;

use jsonwebtoken::EncodingKey;
use sqlx::PgPool;
use warp::Filter;
use warp::reply::json;

use crate::{auth, error, file, gateway, panel, project, upload, user};
use crate::auth::header::with_auth;
use crate::auth::otp::OtpCodes;
use crate::db::with_db;
use crate::models::{CkziuNews, ServerServiceStatus, ServerStatus};
use crate::prelude::web_json;

pub fn routes(key: Arc<EncodingKey>, news: Arc<Vec<CkziuNews>>, otp_codes: OtpCodes, db_pool: PgPool) -> impl Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
  let with_news = warp::any().map(move || Arc::clone(&news));

  let version1 = warp::path!("v1" / ..);

  let status = warp::path!("status").map(|| {
    json(&ServerStatus {
      name: "ckziu-codefest-api".into(),
      author: "Tymon Woźniak".into(),
      version: "0.9".into(),
      services: ServerServiceStatus {
        database: true,
        mail: true,
        login_service: true,
        cez_website: true,
        gateway: false,
      },
    })
  });

  let ckziu_news = warp::path!("ckziu" / "news")
    .and(warp::get())
    .and(with_news.clone())
    .and_then(|news| async move { web_json(&news) });

  let panel = warp::path!("panel")
    .and(warp::get())
    .and(warp::addr::remote())
    .and(with_auth())
    .and(with_db(db_pool.clone()))
    .and_then(panel::api::panel_handler);

  let profile_get = warp::path!("profile" / String)
    .and(warp::get())
    .and(with_auth())
    .and(with_db(db_pool.clone()))
    .and_then(user::api::get_profile);

  let update_user_bio = warp::path!("update" / "user" / "bio")
    .and(warp::post())
    .and(with_auth())
    .and(warp::body::content_length_limit(1024 * 16))
    .and(warp::body::json())
    .and(with_db(db_pool.clone()))
    .and_then(user::api::update_bio);

  let update_user_displayname = warp::path!("update" / "user" / "displayname")
    .and(warp::post())
    .and(with_auth())
    .and(warp::body::content_length_limit(1024 * 16))
    .and(warp::body::json())
    .and(with_db(db_pool.clone()))
    .and_then(user::api::update_displayname);

  let upload_avatar = warp::path!("upload" / "avatar")
    .and(warp::post())
    .and(with_auth())
    .and(warp::multipart::form().max_length(8192 * 1024))
    .and_then(upload::api::upload_profile_picture);

  let get_avatar = warp::path!("avatars" / String)
    .and(warp::get())
    .and(with_db(db_pool.clone()))
    .and_then(file::get_profile);

  // Module routes
  let users = user::routes::routes(&db_pool);
  let projects = project::routes::routes(&db_pool);
  let auth = auth::routes::routes(&db_pool, otp_codes.clone(), key.clone());
  let gateway = gateway::routes::routes();

  let cors = warp::cors()
    .allow_any_origin()
    .allow_headers(vec![
      "User-Agent",
      "Sec-Fetch-Mode",
      "Referer",
      "Origin",
      "Access-Control-Request-Method",
      "Access-Control-Request-Headers",
      "Content-Type",
      "Authorization",
    ])
    .allow_methods(vec!["POST", "GET", "PATCH", "DELETE"]);

  // Combine all routes
  version1
    .and(
      auth.or(projects)
        .or(users)
        .or(panel)
        .or(get_avatar)
        .or(upload_avatar)
        .or(update_user_bio)
        .or(update_user_displayname)
        .or(status)
        .or(ckziu_news)
        .or(profile_get)
    )
    .or(gateway)
    .or(status)
    .recover(error::handle_rejection)
    .with(cors)
  // .with(warp::trace::request())
}