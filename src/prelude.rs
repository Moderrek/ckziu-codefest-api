use serde::Serialize;
use warp::{Rejection, Reply};

pub type WebResult<T> = Result<T, Rejection>;


pub fn web_err<T>(err: crate::error::Error) -> WebResult<T> {
  Err(warp::reject::custom(err))
}

pub fn web_json<T>(val: &T) -> WebResult<impl Reply> where T: Serialize {
  Ok(warp::reply::json(val))
}