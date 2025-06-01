use reqwest::header::{HeaderValue, HeaderMap, CONTENT_TYPE, ACCEPT};
use serde; 
use serde::{Serialize, Deserialize}; 

#[derive(Debug, Clone, Serialize)]
pub struct FinancialHeaderData {
  #[serde(rename = "X-API-KEY")]
  pub api_key : String,
  pub content_type: String
}

#[derive(Debug, Clone, Serialize)]
pub struct LineItemBodyData {
  pub tickers: Vec<String>,
  pub line_items : Vec<String>,
  pub end_date : String, 
  pub period: String,
  pub limit: i64,
}

impl FinancialHeaderData {
  pub fn new(api_key: String) -> Self {
    FinancialHeaderData { api_key: api_key, content_type: "application/json".to_string(), }
  }

  pub fn to_header_map(&self) -> HeaderMap {
    let mut headers: HeaderMap = HeaderMap::new();
    if let Ok(value) = HeaderValue::from_str(&self.api_key) {
      headers.insert("X-API-KEY", value);
    }

    // Add Content-Type header
    if let Ok(value) = HeaderValue::from_str(&self.content_type) {
      headers.insert(CONTENT_TYPE, value);
    }
    
    // Add Accept header
    if let Ok(value) = HeaderValue::from_str("application/json") {
      headers.insert(ACCEPT, value);
    }

    return headers;
  }
}