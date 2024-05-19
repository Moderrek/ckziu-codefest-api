use jsonwebtoken::EncodingKey;
use sqlx::PgPool;
use warp::Filter;

use crate::{auth::header::with_auth, db::with_db, OTPCodes};

use super::api;

pub fn routes(
  db_pool: &PgPool,
  otp_codes: OTPCodes,
  key: EncodingKey,
) -> impl Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
  let otp_codes = warp::any().map(move || otp_codes.clone());
  let jwt_key = warp::any().map(move || key.clone());

  let info = warp::path!("info")
    .and(warp::get())
    .and(with_auth())
    .and(with_db(db_pool.clone()))
    .and_then(api::info);

  let prelogin = warp::path!("prelogin")
    .and(warp::post())
    .and(warp::body::content_length_limit(1024 * 16))
    .and(warp::addr::remote())
    .and(with_db(db_pool.clone()))
    .and(warp::body::json())
    .and_then(api::prelogin);

  let otp = warp::path!("otp")
    .and(warp::post())
    .and(warp::body::content_length_limit(1024 * 16))
    .and(warp::addr::remote())
    .and(warp::body::json())
    .and(otp_codes.clone())
    .and_then(api::auth_otp_handler);

  let register = warp::path!("register")
    .and(warp::addr::remote())
    .and(warp::post())
    .and(warp::body::content_length_limit(1024 * 16))
    .and(otp_codes.clone())
    .and(jwt_key.clone())
    .and(with_db(db_pool.clone()))
    .and(warp::body::json())
    .and_then(api::register);

  let login_credentials = warp::path!("login" / "credentials")
    .and(warp::addr::remote())
    .and(warp::post())
    .and(warp::body::content_length_limit(1024 * 16))
    .and(with_db(db_pool.clone()))
    .and(jwt_key.clone())
    .and(warp::body::json())
    .and_then(api::login_credentials);

  warp::path!("auth" / ..)
    .and(
      prelogin
        .or(otp)
        .or(login_credentials)
        .or(register)
        .or(info)
    )
}
