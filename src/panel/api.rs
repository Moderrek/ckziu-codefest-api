use std::net::SocketAddr;

use serde::Serialize;
use sqlx::PgPool;
use tracing::{info, warn};
use uuid::Uuid;
use warp::{reject, reply::{json, Reply}};

use crate::{error, utils::addr_to_string, WebResult};

#[derive(Serialize)]
struct UserCountResponse {
  usercount: i16,
}

// v1/panel/usercount
pub async fn panel_handler(peer: Option<SocketAddr>, user_uid: Option<Uuid>, db_pool: PgPool) -> WebResult<impl Reply> {
  // Reject unauthorized
  if user_uid.is_none() {
    info!("Peer {} tried to get panel unauthorized.", addr_to_string(&peer));
    return Err(reject::custom(error::Error::NoPermission));
  }
  // Get the user info
  let user_uid = user_uid.unwrap();
  let user = match crate::user::db::get_user_by_id(&user_uid, &db_pool).await {
    Ok(user) => {
      // User Not Found
      if user.is_none() {
        info!("Peer {} ({}) tried to get panel. User Not Found!.", addr_to_string(&peer), &user_uid);
        return Err(reject::custom(error::Error::NoPermission));
      }
      // User Found
      user.unwrap()
    }
    Err(err) => {
      // Query failed
      warn!("Peer {} ({}) tried to get panel. Failed to get user: {}", addr_to_string(&peer), &user_uid, err);
      return Err(reject::custom(error::Error::NoPermission));
    }
  };

  if !user.is_staff() {
    info!("Peer {} ({}) tried to get panel. Not staff (permission int {}).", addr_to_string(&peer), &user_uid, user.id);
    return Err(reject::custom(error::Error::NoPermission));
  }

  let usercount = match crate::user::db::get_userscount(&db_pool).await {
    Ok(usercount) => usercount,
    Err(err) => {
      // Query failed
      warn!("Peer {} {}({}) tried to get panel. Server failed to get usercount: {}", addr_to_string(&peer), &user.name, &user_uid, err);
      return Err(reject::custom(error::Error::ServerProblem));
    }
  };

  info!("Peer {} {}({}) got panel.", addr_to_string(&peer), &user.name, &user.id);
  Ok(json(&UserCountResponse {
    usercount
  }))
}