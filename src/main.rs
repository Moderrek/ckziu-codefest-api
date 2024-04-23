#[macro_use]
extern crate dotenv_codegen;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use dotenv::dotenv;
use log::{error, info, warn};
use reply::json;
use tokio::sync::RwLock;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::{Filter, Rejection, reply, Reply};

use error::Error;

use crate::auth::{auth_exists_handler, auth_login_handler, auth_otp_handler, auth_register_handler, OTPData};
use crate::database::database_handler;
use crate::models::{Article, Project, ServerServiceStatus, ServerStatus, User};
use crate::scrap::async_scrap_cez_news;

mod auth;
mod database;
mod error;
mod mail;
mod models;
mod scrap;
pub mod users;
mod projects;
mod routes;

type Result<T> = std::result::Result<T, Error>;
type WebResult<T> = std::result::Result<T, Rejection>;
type Users = Arc<RwLock<HashMap<String, User>>>;

pub async fn get_articles_handler() -> WebResult<impl Reply> {
  info!("Article sent!");
  Ok(json(&[
    Article::new(
      "CKZiU CodeFest API".into(),
      "Tymon Woźniak".into(),
      "Uruchomienie API!".into(),
    ),
    Article::new(
      "Tytuł".into(),
      "Tymon Woźniak".into(),
      "Opis wspierający UTF-8".into(),
    ),
    Article::new(
      "Testowy artykuł".into(),
      "Tymon Woźniak".into(),
      "Opis wspierający UTF-8".into(),
    ),
    Article::new(
      "Testowy artykuł".into(),
      "Tymon Woźniak".into(),
      "Opis wspierający UTF-8".into(),
    ),
    Article::new(
      "Testowy artykuł".into(),
      "Tymon Woźniak".into(),
      "Opis wspierający UTF-8".into(),
    ),
  ]))
}

pub async fn get_ckziu_news_handler() -> WebResult<impl Reply> {
  info!("Scraping articles!");
  let news = async_scrap_cez_news().await;
  Ok(json(&news))
}

pub async fn trending_projects_handler() -> WebResult<impl Reply> {
  info!("Trending projects!");
  Ok(json(&[
    Project {
      display_name: "Moderrkowo".into(),
      author: "moderr".into(),
      description: "Serwer Minecraft".into(),
      thumbnail_url: "https://static.planetminecraft.com/files/image/minecraft/server/2021/704/14581861-image_l.jpg".into(),
      likes: 0,
    },
    Project {
      display_name: "C-Edit".into(),
      author: "drakvlaa".into(),
      description: "C++ program to make custom cmd shortcut commands".into(),
      thumbnail_url: "https://avatars.githubusercontent.com/u/66324421?v=4".into(),
      likes: 2,
    },
    Project {
      display_name: "KittyCode".into(),
      author: "drakvlaa".into(),
      description: "Edytor kodu uwu".into(),
      thumbnail_url: "https://media.pocketgamer.com/artwork/na-33163-1629209861/Kitty-redeem-codes-header_jpeg_820.jpg".into(),
      likes: 64420420,
    },
    Project {
      display_name: "C-Edit".into(),
      author: "drakvlaa".into(),
      description: "C++ program to make custom cmd shortcut commands".into(),
      thumbnail_url: "https://avatars.githubusercontent.com/u/66324421?v=4".into(),
      likes: 7,
    },
    Project {
      display_name: "C-Edit".into(),
      author: "drakvlaa".into(),
      description: "C++ program to make custom cmd shortcut commands".into(),
      thumbnail_url: "https://avatars.githubusercontent.com/u/66324421?v=4".into(),
      likes: 2,
    },
    Project {
      display_name: "ClaraEngine".into(),
      author: "drakvlaa".into(),
      description: "Silnik 3D do tworzenia gier video na platformę Windows.".into(),
      thumbnail_url: "https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcQz3e4j2AY0Rn7SFpOpQyge9MebJK8BvlI4UhnU9RNgxQ&s".into(),
      likes: 3,
    }
  ]))
}

#[tokio::main]
async fn main() -> Result<()> {
  tracing_subscriber::fmt()
    .with_span_events(FmtSpan::CLOSE)
    .init();

  info!("Starting CKZiU CodeFest Backend API Server");
  info!("Created fully by Tymon Woźniak");
  let working_dir = std::env::current_dir().expect("Failed to load current directory.");
  info!("The current directory is {}", working_dir.display());
  info!("Loading \".env\"");
  dotenv().ok().unwrap();
  let using_tls: bool = dotenv!("USE_TLS") == "true";
  let cert_path = dotenv!("CERT_PATH");
  let key_path = dotenv!("KEY_PATH");
  if using_tls {
    info!("Using TLS");
    let mut success = true;
    if !Path::new(cert_path).exists() {
      error!(
                "Cannot find certificate file! @ {}",
                Path::new(cert_path).display()
            );
      success = false;
    }
    if !Path::new(key_path).exists() {
      error!("Cannot find key file! @ {}", Path::new(key_path).display());
      success = false;
    }
    if !success {
      return Err(Error::CannotFindFile);
    }
    info!("Cert: {}; Key: {}", cert_path, key_path);
  } else {
    warn!("Server is not using TLS!");
  }
  let domain = dotenv!("DOMAIN");
  let port = dotenv!("PORT");
  info!(
        "API URL: {}://{}:{}",
        if using_tls { "https" } else { "http" },
        domain,
        port
    );
  let address: SocketAddr = format!("{domain}:{port}")
    .parse()
    .expect("Failed to parse address.");

  info!("Init database pool..");
  let db_pool = database::create_pool().await.unwrap();
  // db_insert_user(&CodeFestUser {
  //   name: "moderr".to_string(),
  //   display_name: "Moderr".to_string(),
  //   id: Uuid::new_v4(),
  //   bio: None,
  //   created_at: Utc::now(),
  //   updated_at: Utc::now(),
  //   flags: 0,
  // }, &db_pool).await.unwrap();
  // db_insert_user(&CodeFestUser {
  //   name: "drakvlaa".to_string(),
  //   display_name: "Dr Akula".to_string(),
  //   id: Uuid::new_v4(),
  //   bio: None,
  //   created_at: Utc::now(),
  //   updated_at: Utc::now(),
  //   flags: 0,
  // }, &db_pool).await.unwrap();
  let with_db = warp::any().map(move || db_pool.clone());

  let users: Users = Arc::new(RwLock::new(HashMap::new()));
  let _users = warp::any().map(move || users.clone());

  let otp_codes: Arc<RwLock<HashMap<String, OTPData>>> = Arc::new(RwLock::new(HashMap::new()));
  let otp_codes = warp::any().map(move || otp_codes.clone());

  let version1 = warp::path!("v1" / ..);

  let status = warp::path!("status").map(|| {
    json(&ServerStatus {
      name: "ckziu-codefest-api".into(),
      author: "Tymon Woźniak".into(),
      version: "dev-0.1".into(),
      services: ServerServiceStatus {
        database: false,
        mail: false,
        login_service: true,
        cez_website: false,
        gateway: false,
      },
    })
  });

  let articles = warp::path!("article")
    .and(warp::get())
    .and_then(get_articles_handler);

  let ckziu_news = warp::path!("ckziu" / "news")
    .and(warp::get())
    .and_then(get_ckziu_news_handler);

  let trending_projects = warp::path!("trending" / "projects")
    .and(warp::get())
    .and_then(trending_projects_handler);

  let auth = warp::path!("auth" / ..);

  let auth_otp = warp::path!("otp")
    .and(warp::post())
    .and(warp::body::json())
    .and(otp_codes.clone())
    .and_then(auth_otp_handler);

  let auth_login = warp::path!("login")
    .and(warp::post())
    .and(warp::body::json())
    .and(otp_codes)
    .and_then(auth_login_handler);

  let auth_exists = warp::path!("exists" / String)
    .and(with_db.clone())
    .and_then(auth_exists_handler);

  let auth_register = warp::path!("register")
    .and(warp::post())
    .and(warp::body::json())
    .and(with_db.clone())
    .and_then(auth_register_handler);

  let cors = warp::cors()
    .allow_any_origin()
    .allow_headers(vec![
      "User-Agent",
      "Sec-Fetch-Mode",
      "Referer",
      "Origin",
      "Access-Control-Request-Method",
      "Access-Control-Request-Headers",
      "Content-Type",
    ])
    .allow_methods(vec!["POST", "GET"]);

  let options_route =
    warp::options().map(|| reply::with_header("OK", "Access-Control-Allow-Origin", "*"));

  let db_test = warp::path!("dbtest")
    .and(with_db.clone())
    .and_then(database_handler);

  let users_get = warp::path!("users" / String)
    .and(with_db.clone())
    .and_then(users::api_get_user);

  let routes = version1
    .and(
      auth.and(auth_login.or(auth_register).or(auth_otp).or(auth_exists))
        .or(db_test)
        .or(status)
        .or(ckziu_news)
        .or(articles)
        .or(options_route)
        .or(trending_projects)
        .or(users_get),
    )
    .or(status)
    .recover(error::handle_rejection)
    .with(cors);

  info!("Created routes");

  match using_tls {
    true => {
      info!("Serving with TLS..");
      warp::serve(routes)
        .tls()
        .cert_path(cert_path)
        .key_path(key_path)
        .run(address)
        .await;
    }
    false => {
      info!("Serving..");
      warp::serve(routes).run(address).await;
    }
  }

  info!("Bye");

  Ok(())
}
