#[macro_use]
extern crate dotenv_codegen;

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use dotenv::dotenv;
use jsonwebtoken::EncodingKey;
use tracing::{error, info, warn};
use tracing_subscriber::fmt::format::FmtSpan;

use error::Error;

use crate::auth::otp;
use crate::prelude::WebResult;
use crate::scrap::scrap_news;

mod auth;
mod cache;
mod db;
mod error;
mod file;
mod gateway;
mod mail;
mod models;
mod panel;
pub mod posts;
mod prelude;
mod project;
mod routes;
mod scrap;
mod upload;
mod user;
mod utils;

#[cfg(test)]
mod test;

#[tokio::main]
async fn main() -> Result<(), error::Error> {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();

    info!("Starting CKZiU CodeFest Backend API Server v0.9");
    info!("Created fully by Tymon Woźniak @Moderrek <tymon.student@gmail.com>");
    let working_dir = std::env::current_dir().expect("Failed to load current directory.");
    info!("The current directory is {}", working_dir.display());
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
    let address: SocketAddr = format!("{domain}:{port}")
        .parse()
        .expect("Failed to parse address.");
    info!(
        "API URL: {}://{}:{}",
        if using_tls { "https" } else { "http" },
        domain,
        port
    );

    info!("Init db pool..");
    let otp_codes = otp::create_otp_memory();
    let key = Arc::new(EncodingKey::from_secret(dotenv!("TOKEN_SECRET").as_bytes()));
    let db_pool = db::create_pool().await.unwrap();
    let news = Arc::new(scrap_news().await.unwrap());

    let routes = routes::routes(key, news, otp_codes, db_pool);
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
