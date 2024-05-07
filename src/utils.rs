use std::net::SocketAddr;

pub fn addr_to_string(addr: &Option<SocketAddr>) -> String {
  match addr {
    Some(addr) => addr.to_string(),
    None => "Unknown".into(),
  }
}
