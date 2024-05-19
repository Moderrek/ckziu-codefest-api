use warp::Filter;

use super::handler::handle_client;

pub fn routes() -> impl Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
  let gateway = warp::path("gateway")
    .and(warp::ws())
    .map(|ws: warp::ws::Ws| {
      ws.on_upgrade(handle_client)
    });

  gateway
}
