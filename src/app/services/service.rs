use super::agent_service::AgentService;
use crate::ai_agent::utils::analysts::get_analyst_order;
use crate::ai_agent::llm::models::{get_available_models, get_ollama_models};

use std::collections::HashMap;
use chrono::{NaiveDate, Local};
use serde_json::Value;
use anyhow::{Error, Ok};
use std::result::Result;
use std::option::Option;


pub struct HedgeFundServices {
  agent_service : AgentService
}

impl HedgeFundServices {

  pub fn new(agent_service: AgentService) -> Self {
    HedgeFundServices { agent_service: agent_service }
  }

  pub fn get_available_models(&self) -> Result<(Vec<HashMap<String, String>>, Vec<HashMap<String, String>>), Error> {
    let standard_models = get_available_models().iter().map(|model| {
      let mut map = HashMap::new(); 
      map.insert("display_name".to_string(), model.display_name.clone());
      map.insert("model_name".to_string(), model.model_name.clone());
      map.insert("provider".to_string(), model.provider.to_string());
      map
    }).collect();

    let ollama_models = get_ollama_models().iter().map(|model| {
      let mut map = HashMap::new(); 
      map.insert("display_name".to_string(), model.display_name.clone());
      map.insert("model_name".to_string(), model.model_name.clone());
      map.insert("provider".to_string(), model.provider.to_string());
      map
    }).collect();

    return Ok((standard_models, ollama_models));
  }

  pub fn get_available_analysts(&self) -> Result<Vec<HashMap<String, String>>, Error> {
    let analysts = get_analyst_order().iter().map(|(display_name, key)| {
      let mut map = HashMap::new(); 
      map.insert("display_name".to_string(), display_name.to_string());
      map.insert("key".to_string(), key.to_string());
      map
    }).collect();

    return Ok(analysts);
  }


  pub async fn hedge_fund(&self, tickers: Vec<String>, start_date: Option<&str>, end_date: Option<&str>, 
                          initial_cash: Option<f64>, margin_requirement: Option<f64>, show_reasoning: Option<bool>, 
                          selected_analysts: Option<Vec<String>>, model_name: Option<String>, model_provider: Option<String>) -> Result<HashMap<String, Value>, Error> {
    
    let initial_cash: f64 = initial_cash.unwrap_or(100000.0);
    let margin_requirement: f64 = margin_requirement.unwrap_or(0.0); 


    let end_date: String = match end_date {
      Some(date) => date.to_string(), 
      None => Local::now().format("%Y-%m-%d").to_string(),
    };

    let start_date: String = match start_date {
      Some(date) => date.to_string(), 
      None => {
        let end_date_obj: NaiveDate = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
                    .unwrap_or_else(|_| Local::now().naive_local().date());
        let start_date_obj: NaiveDate = end_date_obj - chrono::Duration::days(90); // Approximately 3 months
        start_date_obj.format("%Y-%m-%d").to_string()
      }
    };

    let mut portfolio = HashMap::new(); 
    portfolio.insert("cash".to_string(), Value::from(initial_cash)); 
    portfolio.insert("margin_requirement".to_string(), Value::from(margin_requirement)); 
    portfolio.insert("margin_used".to_string(), Value::from(0.0)); 

    let mut positions: HashMap<String, Value> = HashMap::new(); 
    for ticker in &tickers {
      let mut position: HashMap<String, Value> = HashMap::new(); 
      position.insert("long".to_string(), Value::from(0)); 
      position.insert("short".to_string(), Value::from(0));
      position.insert("long_cost_basis".to_string(), Value::from(0.0)); 
      position.insert("short_cost_basis".to_string(), Value::from(0.0)); 
      position.insert("short_margin_used".to_string(), Value::from(0.0)); 
      positions.insert(ticker.clone(), Value::Object(position.into_iter().collect())); 
    }

    portfolio.insert("positions".to_string(), Value::Object(positions.into_iter().collect())); 

    let mut realized_gains: HashMap<String, Value> = HashMap::new();
    for ticker in &tickers {
      let mut gains : HashMap<String, Value> = HashMap::new(); 
      gains.insert("long".to_string(), Value::from(0.0)); 
      gains.insert("short".to_string(), Value::from(0.0)); 
      realized_gains.insert("realized_gains".to_string(), Value::Object(gains.into_iter().collect())); 
    }

    portfolio.insert("realized_gains".to_string(), Value::Object(realized_gains.into_iter().collect())); 
  
    return self.agent_service.run_hedge_fund(
      tickers,
      &start_date,
      &end_date,
      portfolio,
      show_reasoning,
      selected_analysts,
      model_name.as_deref(),
      model_provider.as_deref(),
    ).await;
  }

}