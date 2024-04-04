use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::Duration;
use log::info;
use reply::json;
use serde::{Deserialize, Serialize};
use tracing_subscriber::fmt::format::FmtSpan;
use warp::{Filter, Rejection, reply};

// type Result<T> = std::result::Result<T, error::Error>;
// type WebResult<T> = std::result::Result<T, Rejection>;

#[derive(Serialize)]
struct ServerServiceStatus {
  login_service: bool,
}

#[derive(Serialize)]
struct ServerStatus {
  name: String,
  version: String,
  services: ServerServiceStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
  pub uid: String,
  pub name: String,
  pub role: u64,
}

#[derive(Deserialize)]
pub struct LoginRequest {
  pub email: String
}

#[derive(Serialize)]
pub struct LoginResponse {
  pub token: String
}

#[derive(Serialize)]
struct Article {
  pub title: String,
  pub author: String,
  pub description: String
}

impl Article {
  fn new(title: String, author: String, description: String) -> Self {
    Self {
      title,
      author,
      description
    }
  }
}

pub async fn create_todo_handler() -> Result<impl warp::Reply, Infallible> {
  info!("Article sent!");
  tokio::time::sleep(Duration::from_secs(5)).await;
  Ok(json(&[
    Article::new("CKZiU CodeFest API".into(), "Tymon Woźniak".into(), "Uruchomienie API!".into()),
    Article::new("Tytuł".into(), "Tymon Woźniak".into(), "Opis wspierający UTF-8".into()),
    Article::new("Testowy artytuł".into(), "Tymon Woźniak".into(), "Opis wspierający UTF-8".into()),
    Article::new("Testowy artytuł".into(), "Tymon Woźniak".into(), "Opis wspierający UTF-8".into()),
    Article::new("Testowy artytuł".into(), "Tymon Woźniak".into(), "Opis wspierający UTF-8".into())
  ]))
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // env_logger::init();
  let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "tracing=info,warp=debug".to_owned());

  tracing_subscriber::fmt()
    .with_env_filter(filter)
    .with_span_events(FmtSpan::CLOSE)
    .init();

  let address: SocketAddr = "25.50.65.38:3030".parse().expect("Failed to parse address.");

  let cors = warp::cors()
    .allow_any_origin();

  // GET /hello/warp => 200 OK with body "Hello, warp!"
  let hello = warp::path!("hello" / String)
    .map(|name| format!("Hello, {}!", name));

  let status = warp::path!("status")
    .map(|| json(&ServerStatus {
      name: "ckziu-codefest-api".into(),
      version: "dev-0.1".into(),
      services: ServerServiceStatus {
        login_service: false
      },
    }));

  let articles = warp::path!("article")
    .and_then(create_todo_handler);

  let auth = warp::path!("auth")
    .map(|| {

    });

  let routes = warp::get()
    .and(
      hello
        .or(status)
        .or(articles)
    )
    .with(warp::cors().allow_any_origin());

  warp::serve(routes)
    .run(address)
    .await;

  Ok(())
}
