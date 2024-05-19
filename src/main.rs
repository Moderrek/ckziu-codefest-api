#[macro_use]
extern crate dotenv_codegen;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use dotenv::dotenv;
use jsonwebtoken::EncodingKey;
use log::{error, info, warn};
use reply::json;
use tokio::sync::RwLock;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::{Filter, Rejection, reply, Reply};

use error::Error;

use crate::auth::header::with_auth;
use crate::auth::otp::Otp;
use crate::db::with_db;
use crate::models::{ServerServiceStatus, ServerStatus};
use crate::scrap::scrap_news;

use self::models::CkziuNews;

mod auth;
mod cache;
mod db;
mod error;
mod file;
mod mail;
mod models;
mod panel;
mod project;
mod scrap;
mod upload;
mod user;
mod utils;
mod gateway;
mod prelude;

#[cfg(test)]
mod test;

type Result<T> = std::result::Result<T, Error>;
type WebResult<T> = std::result::Result<T, Rejection>;

async fn get_ckziu_news_handler(news: Vec<CkziuNews>) -> WebResult<impl Reply> {
  Ok(json(&news))
}

pub type OTPCodes = Arc<RwLock<HashMap<String, Otp>>>;

#[tokio::main]
async fn main() -> Result<()> {
  tracing_subscriber::fmt()
    .with_span_events(FmtSpan::CLOSE)
    .init();

  info!("Starting CKZiU CodeFest Backend API Server v0.9");
  info!("Created fully by Tymon Woźniak @Moderrek <tymon.student@gmail.com>");
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

  info!("Init db pool..");
  let db_pool = db::create_pool().await.unwrap();

  let otp_codes: OTPCodes = Arc::new(RwLock::new(HashMap::new()));
  let key = EncodingKey::from_secret(dotenv!("TOKEN_SECRET").as_bytes());

  let news = scrap_news().await.unwrap();
  let news = warp::any().map(move || news.clone());

  let version1 = warp::path!("v1" / ..);

  let status = warp::path!("status").map(|| {
    json(&ServerStatus {
      name: "ckziu-codefest-api".into(),
      author: "Tymon Woźniak".into(),
      version: "0.9".into(),
      services: ServerServiceStatus {
        database: true,
        mail: true,
        login_service: true,
        cez_website: true,
        gateway: false,
      },
    })
  });

  let ckziu_news = warp::path!("ckziu" / "news")
    .and(warp::get())
    .and(news.clone())
    .and_then(get_ckziu_news_handler);

  let panel = warp::path!("panel")
    .and(warp::get())
    .and(warp::addr::remote())
    .and(with_auth())
    .and(with_db(db_pool.clone()))
    .and_then(panel::api::panel_handler);

  let profile_get = warp::path!("profile" / String)
    .and(warp::get())
    .and(with_auth())
    .and(with_db(db_pool.clone()))
    .and_then(user::api::get_profile);

  let update_user_bio = warp::path!("update" / "user" / "bio")
    .and(warp::post())
    .and(auth::header::with_auth())
    .and(warp::body::content_length_limit(1024 * 16))
    .and(warp::body::json())
    .and(with_db(db_pool.clone()))
    .and_then(user::api::update_bio);

  let update_user_displayname = warp::path!("update" / "user" / "displayname")
    .and(warp::post())
    .and(with_auth())
    .and(warp::body::content_length_limit(1024 * 16))
    .and(warp::body::json())
    .and(with_db(db_pool.clone()))
    .and_then(user::api::update_displayname);

  let upload_avatar = warp::path!("upload" / "avatar")
    .and(warp::post())
    .and(with_auth())
    .and(warp::multipart::form().max_length(8192 * 1024))
    .and_then(upload::api::upload_profile_picture);

  let get_avatar = warp::path!("avatars" / String)
    .and(warp::get())
    .and(with_db(db_pool.clone()))
    .and_then(file::get_profile);

  // Module routes
  let users = user::routes::routes(&db_pool);
  let projects = project::routes::routes(&db_pool);
  let auth = auth::routes::routes(&db_pool, otp_codes.clone(), key.clone());
  let gateway = gateway::routes::routes();

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
      "Authorization",
    ])
    .allow_methods(vec!["POST", "GET", "PATCH", "DELETE"]);

  // Combine all routes
  let routes =
    version1
      .and(
        auth.or(projects)
          .or(users)
          .or(panel)
          .or(get_avatar)
          .or(upload_avatar)
          .or(update_user_bio)
          .or(update_user_displayname)
          .or(status)
          .or(ckziu_news)
          .or(profile_get)
      )
      .or(gateway)
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
    _ => {
      info!("Serving..");
      warp::serve(routes).run(address).await;
    }
  }

  info!("Bye");
  Ok(())
}
