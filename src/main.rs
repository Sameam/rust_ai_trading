use actix_web::HttpServer;
use std::env; 

use crate::app::config::Config;
use crate::app::factory::CreateApp;

mod app; 
mod ai_agent;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  if env::var_os("RUST_LOG").is_none() {
    env::set_var("RUST_LOG", "actix_web=debug,debug"); // Default to info for actix_web and your app
  }
  env_logger::init();

  dotenv::dotenv().ok();

  let config : Config = Config::load();

  let server_builder = HttpServer::new(move || {
    let factory: CreateApp = CreateApp::new(config.clone());
    factory.build_app().wrap(actix_web::middleware::Logger::default())
  });

  let server = server_builder.bind(("127.0.0.1", 8080))?;

  server.run().await?;

  Ok(())
}