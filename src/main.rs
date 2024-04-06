use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::Duration;
use log::info;
use reply::json;
use scraper::{Html, Selector};
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

pub async fn get_articles_handler() -> Result<impl warp::Reply, Infallible> {
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

pub async fn get_ckziu_news_handler() -> Result<impl warp::Reply, Infallible> {
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


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "tracing=info,warp=debug".to_owned());

  tracing_subscriber::fmt()
    .with_env_filter(filter)
    .with_span_events(FmtSpan::CLOSE)
    .init();

  let address: SocketAddr = "25.50.65.38:3030".parse().expect("Failed to parse address.");

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
    .map(|| {

    });

  let routes = warp::get()
    .and(
      hello
        .or(status)
        .or(articles)
        .or(ckziu_news)
    )
    .with(warp::cors().allow_any_origin());

  warp::serve(routes)
    .run(address)
    .await;

  Ok(())
}
