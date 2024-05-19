use serde::{Deserialize, Serialize};

use super::opcode::Opcode;

#[derive(Debug, Serialize)]
pub struct WebSocketMessage {
  #[serde(rename = "o")]
  pub opcode: Opcode,
  #[serde(rename = "d")]
  pub data: WebSocketData,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum WebSocketData {
  Identify {
    token: String
  }
}
