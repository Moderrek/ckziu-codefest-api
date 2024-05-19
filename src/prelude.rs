use warp::Rejection;

pub fn web_err<T>(err: crate::error::Error) -> Result<T, Rejection> {
  Err(warp::reject::custom(err))
}