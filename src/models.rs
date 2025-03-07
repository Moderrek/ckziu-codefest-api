use serde::Serialize;

#[derive(Serialize)]
pub struct Project {
  pub display_name: String,
  pub author: String,
  pub description: String,
  pub thumbnail_url: String,
  pub likes: usize,
}

#[derive(Serialize)]
pub struct ServerServiceStatus {
  pub login_service: bool,
  pub database: bool,
  pub mail: bool,
  pub cez_website: bool,
  pub gateway: bool,
}

#[derive(Serialize)]
pub struct ServerStatus {
  pub name: String,
  pub author: String,
  pub version: String,
  pub services: ServerServiceStatus,
}

#[derive(Serialize)]
pub struct Article {
  pub title: String,
  pub author: String,
  pub description: String,
}

#[derive(Serialize, Clone)]
pub struct CkziuNews {
  pub title: String,
  pub description: String,
  pub url: String,
}
