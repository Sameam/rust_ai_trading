use std::env;

use log; 

#[allow(unused)]
#[derive(Clone)]
pub struct Config {
  pub antropic_api_key: String,
  pub deepseek_api_key: String,
  pub groq_api_key : String, 
  pub google_api_key : String, 
  pub financial_datasets_api_key : String,
  pub openai_api_key : String,
}

impl Config {

  #[allow(unused)]
  pub fn load() -> Self {
    match dotenv::dotenv() {
      Ok(_) => log::info!("Loaded .env file"),
      Err(_) => log::error!("No .env file found"),
    }

    let antropic_api_key: String =  env::var("ANTHROPIC_API_KEY").unwrap_or_else(|_| {
      log::error!("Warning: anthropic api key not found, using default http://localhost:8000");
      "http://localhost:8000".to_string()
    });
    let deepseek_api_key : String = env::var("DEEPSEEK_API_KEY").unwrap_or_else(|_| {
      log::error!("Warning: TTS_URL not found, using default http://localhost:8000");
      "ws://localhost:8000".to_string()
    });
    let groq_api_key : String = env::var("GROQ_API_KEY").unwrap_or_else(|_| {
      log::error!("Warning: TTS_URL not found, using default http://localhost:8000");
      "ws://localhost:8000".to_string()
    });

    let google_api_key : String = env::var("GOOGLE_API_KEY").unwrap_or_else(|_| {
      log::error!("Warning: TTS_URL not found, using default http://localhost:8000");
      "ws://localhost:8000".to_string()
    });

    let financial_datasets_api_key : String = env::var("FINANCIAL_DATASETS_API_KEY").unwrap_or_else(|_| {
      log::error!("Warning: TTS_URL not found, using default http://localhost:8000");
      "ws://localhost:8000".to_string()
    });

    let openai_api_key : String =  env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
      log::error!("Warning: TTS_URL not found, using default http://localhost:8000");
      "ws://localhost:8000".to_string()
    });
    

    return Config {
      antropic_api_key, deepseek_api_key, groq_api_key, google_api_key, financial_datasets_api_key, openai_api_key
    }
  }

}