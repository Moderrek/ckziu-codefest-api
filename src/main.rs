mod error;

use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use chrono::Utc;
use jsonwebtoken::{Algorithm, encode, EncodingKey, Header};
use log::info;
use reply::json;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing_subscriber::fmt::format;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::{Filter, Rejection, reply, Reply};

type Result<T> = std::result::Result<T, error::Error>;
type WebResult<T> = std::result::Result<T, Rejection>;
type Users = Arc<RwLock<HashMap<String, User>>>;

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
}

#[derive(Deserialize)]
pub struct LoginRequest {
  pub email: String
}

#[derive(Serialize)]
pub struct LoginResponse {
  pub token: Option<String>,
  pub message: String,
}

#[derive(Serialize)]
struct Article {
  pub title: String,
  pub author: String,
  pub description: String
}

#[derive(Serialize)]
struct CkziuNews {
  title: String,
  description: String,
  url: String
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

pub async fn get_articles_handler() -> WebResult<impl Reply> {
  info!("Article sent!");
  Ok(json(&[
    Article::new("CKZiU CodeFest API".into(), "Tymon Woźniak".into(), "Uruchomienie API!".into()),
    Article::new("Tytuł".into(), "Tymon Woźniak".into(), "Opis wspierający UTF-8".into()),
    Article::new("Testowy artytuł".into(), "Tymon Woźniak".into(), "Opis wspierający UTF-8".into()),
    Article::new("Testowy artytuł".into(), "Tymon Woźniak".into(), "Opis wspierający UTF-8".into()),
    Article::new("Testowy artytuł".into(), "Tymon Woźniak".into(), "Opis wspierający UTF-8".into())
  ]))
}

pub async fn get_ckziu_news_handler() -> WebResult<impl Reply> {
  let response = reqwest::get("https://cez.lodz.pl").await.unwrap();
  let html_content = response.text().await.unwrap();
  let document = Html::parse_document(&html_content);
  let news_selector = Selector::parse("div.event-post").unwrap();
  let all_news = document.select(&news_selector);

  let mut parsed_news: Vec<CkziuNews> = Vec::new();

  for news in all_news {
    let a = news.select(&Selector::parse("a").unwrap()).next().unwrap();

    let url: String = a.value().attr("href").unwrap().into();
    let title: String = a.text().next().unwrap().into();

    let p = news.select(&Selector::parse("p").unwrap()).next().unwrap();
    let description: String = p.text().next().unwrap().into();

    parsed_news.push(CkziuNews {
      title,
      url,
      description
    });
  }

  Ok(json(&parsed_news))
}

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
  uid: String,
  role: String,
  exp: usize,
}

pub fn create_jwt(uid: &str, role: &str) -> Result<String> {
  let expiration = Utc::now()
    .checked_add_signed(chrono::Duration::seconds(60))
    .expect("Valid timestamp")
    .timestamp();

  let claims = Claims {
    uid: uid.into(),
    role: role.into(),
    exp: expiration as usize
  };

  let header = Header::new(Algorithm::HS512);
  Ok(encode(&header, &claims, &EncodingKey::from_secret(b"secret")).unwrap())
}

pub async fn auth_handler(body: LoginRequest) -> WebResult<impl Reply> {
  tokio::time::sleep(Duration::from_secs(5)).await;
  println!("Email: {}", body.email);
  if !body.email.ends_with("ckziu.elodz.edu.pl") {
    // return Err(warp::reject::custom(error::Error::WrongCredentialsError));
    return Ok(json(
      &LoginResponse {
        token: None,
        message: "Nieprawidłowy email".into()
      }
    ));
  }
  Ok(json(&LoginResponse{
    token: Some(create_jwt(body.email.as_str(), "user").unwrap()),
    message: "Pomyślnie zalogowano".into()
  }))
}

#[tokio::main]
async fn main() -> Result<()> {
  let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "tracing=debug,warp=debug".to_owned());

  tracing_subscriber::fmt()
    .with_env_filter(filter)
    .with_span_events(FmtSpan::CLOSE)
    .init();

  let address: SocketAddr = "25.50.65.38:8080".parse().expect("Failed to parse address.");

  let pool = sqlx::postgres::PgPoolOptions::new()
    .max_connections(1)
    .connect("postgres://avnadmin:AVNS_vcH6CYuY4vN7Ayg8DoB@pg-1c46544a-tymonek12345-153d.a.aivencloud.com:25654/defaultdb?sslmode=require").await.unwrap();

  let row: (i64,) = sqlx::query_as("SELECT $1")
    .bind(150_i64)
    .fetch_one(&pool).await.unwrap();

  println!("{:?}", row);

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
    .and_then(get_articles_handler);

  let ckziu_news = warp::path!("ckziu" / "news")
    .and_then(get_ckziu_news_handler);

  let auth = warp::path!("auth")
    .and(warp::post())
    .and(warp::body::json())
    .and_then(auth_handler);

  let cors = warp::cors()
    .allow_any_origin()
    .allow_headers(vec![
      "User-Agent", "Sec-Fetch-Mode", "Referer", "Origin",
      "Access-Control-Request-Method", "Access-Control-Request-Headers",
      "Content-Type"
    ])
    .allow_methods(vec!["POST", "GET"]);

  let options_route = warp::options()
    .map(|| warp::reply::with_header("OK", "Access-Control-Allow-Origin", "*"));

  let routes = auth.or(articles).or(options_route).with(cors);
  warp::serve(routes)
    .run(address)
    .await;

  Ok(())
}
