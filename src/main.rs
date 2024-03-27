use tide::prelude::json;

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
  // TODO read from .env or default
  let host = "0.0.0.0";
  let port = 8080;

  // Init global tracing subscriber
  tracing_subscriber::fmt::init();


  let mut server = tide::Server::new();
  server.with(tide::log::LogMiddleware::new());

  server.at("/").get(|_| async { Ok("Hello, world!") });
  server.at("/info").get(|_| async {
    Ok(json!({
            "version": "test",
            "status": "operational"
        }))
  });

  server.listen((host, port)).await?;
  Ok(())
}