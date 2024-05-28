use futures::{stream::SplitSink, StreamExt};
use tracing::{error, info};
use warp::filters::ws::{Message, WebSocket};

pub async fn handle_client(websocket: warp::ws::WebSocket) {
  info!("WebSocket connected");

  let (mut sender, mut receiver) = websocket.split();

  while let Some(body) = receiver.next().await {
    let message = match body {
      Ok(msg) => msg,
      Err(err) => {
        error!("WebSocket failed read message on: {err}");
        break;
      }
    };

    handle_message(message, &mut sender).await;
  }
  info!("WebSocket disconnected");
}

async fn handle_message(message: Message, _sender: &mut SplitSink<WebSocket, Message>) {
  let msg = if let Ok(str) = message.to_str() {
    str
  } else { return; };
  info!("WebSocket received {msg}");
}
