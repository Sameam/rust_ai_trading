use anyhow::Error;
use serde_json:: Value;
use std::collections::HashMap; 
use std::result::Result::{Ok};
use std::future::Future; 
use std::pin::Pin;


use crate::ai_agent::graph::state::{AgentState, PartialAgentStateUpdate, show_agent_reasoning}; 
use crate::ai_agent::llm::model_provider::ChatMessage;
use crate::ai_agent::tools::api::API;
use crate::app::config::Config;

pub struct RiskManagerAgent;

impl RiskManagerAgent {
  pub fn new() -> Self {
    RiskManagerAgent {}
  } 

  pub fn static_risk_management_agent(state: AgentState, config: Config) -> Pin<Box<dyn Future<Output = Result<PartialAgentStateUpdate, Error>> + Send>> {
    Box::pin(async move {
      let risk_management = RiskManagerAgent::new();
      risk_management.risk_management_agent(state, config).await
    })
  }

  pub async fn risk_management_agent(&self, state: AgentState, config: Config) -> Result<PartialAgentStateUpdate, Error> {
    /* Controls position sizing based on real_world risk factors for multiple tickers
     */

    let api = API::new(config);

    let portfolio = match state.data.get("portfolio") {
      Some(portfolio) => portfolio, 
      _ => {
        log::error!("Cannot find portfolio inside state.data"); 
        return Ok(PartialAgentStateUpdate::new());
      }
    }; 

    let data: HashMap<String, Value> = state.data.clone();
    let tickers: Vec<String> = match data.get("tickers").and_then(Value::as_array) {
      Some(arr) if !arr.is_empty() => {
        arr.iter().filter_map(Value::as_str).map(String::from).collect()
      }
      _ => {
        log::error!("Invalid end_date format for market cap");
        return Ok(PartialAgentStateUpdate::new());
      }
    };

    let start_date: &str = match data.get("start_date").and_then(Value::as_str) {
      Some(start_date) => start_date, 
      _ => {
        log::error!("Cannot find start date inside state.data"); 
        return Ok(PartialAgentStateUpdate::new());
      }
    }; 

    let end_date: &str = match data.get("end_date").and_then(Value::as_str) {
      Some(end_date) => end_date, 
      _ => {
        log::error!("Cannot find start date inside state.data"); 
        return Ok(PartialAgentStateUpdate::new());
      }
    }; 

    let mut risk_analysis : HashMap<String, Value> = HashMap::new();
    let mut current_prices : HashMap<String, f64> = HashMap::new();

    for ticker in tickers {
      let prices = api.get_price(&ticker, start_date, end_date).await?; 

      if prices.is_empty() {
        log::info!("Risk management agent, {}, Failed no price data found", ticker); 
        continue;
      }

      let prices_df = match api.prices_to_df(prices) {
        Ok(df) => df, 
        Err(e) => {
          log::error!("Failed to convert prices to DataFrame for {}: {}", ticker, e);
          continue;
        }
      };

      let current_price = match prices_df.column("close") {
        Ok(column) => {
          let len = column.len(); 
          if len == 0 {
            log::error!("No close prices available for {}", ticker);
            continue;
          }
          match column.get(len - 1) {
            Ok(value) => match value.try_extract::<f64>() {
              Ok(price) => price,
              Err(e) => {
                log::error!("Failed to extract close price for {} with error: {}", ticker, e);
                continue;
              }
            }
            Err(e) => {
              log::error!("Failed to get last close price for {}: {}", ticker, e);
              continue;
            }
          }
        }
        Err(e) => {
          log::error!("Failed to get close column for {}: {}", ticker, e);
          continue;
        }

      }; 

      current_prices.insert(ticker.clone(), current_price); 

      let current_position_value = portfolio.get("cost_basis").and_then(|cost_basis| cost_basis.get(&ticker)).and_then(Value::as_f64).unwrap_or(0.0);

      let portfolio_cash = portfolio.get("cash").and_then(Value::as_f64).unwrap_or(0.0);
      
      let mut total_portfolio_value = portfolio_cash;
      
      if let Some(cost_basis) = portfolio.get("cost_basis").and_then(Value::as_object) {
        for (_, value) in cost_basis {
          if let Some(position_value) = value.as_f64() {
            total_portfolio_value += position_value;
          }
        }
      }

      let position_limit = total_portfolio_value * 0.20; 

      let remaining_position_limit = position_limit - current_position_value; 

      let max_position_size = remaining_position_limit.min(portfolio_cash); 


      // Create risk analysis entry for this ticker
      let mut ticker_analysis = HashMap::new();
      ticker_analysis.insert("remaining_position_limit".to_string(), Value::from(max_position_size));
      ticker_analysis.insert("current_price".to_string(), Value::from(current_price));
      
      // Add reasoning
      let mut reasoning = HashMap::new();
      reasoning.insert("portfolio_value".to_string(), Value::from(total_portfolio_value));
      reasoning.insert("current_position".to_string(), Value::from(current_position_value));
      reasoning.insert("position_limit".to_string(), Value::from(position_limit));
      reasoning.insert("remaining_limit".to_string(), Value::from(remaining_position_limit));
      reasoning.insert("available_cash".to_string(), Value::from(portfolio_cash));
      
      ticker_analysis.insert("reasoning".to_string(), Value::Object(reasoning.into_iter().collect()));
      
      // Add to risk analysis
      risk_analysis.insert(ticker.clone(), Value::Object(ticker_analysis.into_iter().collect()));
    }

    let message_content = serde_json::to_string(&risk_analysis); 

    let message = ChatMessage { role: "assistant".to_string(), content: message_content?};

     // Show reasoning if requested
    if let Some(show_reasoning) = state.metadata.get("show_reasoning").and_then(Value::as_bool) {
      if show_reasoning {
        show_agent_reasoning(&serde_json::to_string(&risk_analysis)?, "Risk Management Agent");
      }
    }

    // Create partial state update
    let mut updated_data = data.clone();
    
    // Get or create analyst_signals
    let analyst_signals = match updated_data.get_mut("analyst_signals") {
      Some(Value::Object(signals)) => signals,
      _ => {
        updated_data.insert("analyst_signals".to_string(), Value::Object(HashMap::new().into_iter().collect()));
        match updated_data.get_mut("analyst_signals") {
          Some(Value::Object(signals)) => signals,
          _ => {
            log::error!("Failed to create analyst_signals in data");
            return Ok(PartialAgentStateUpdate::new());
          }
        }
      }
    };
    
    // Add risk analysis to analyst_signals
    analyst_signals.insert("risk_management_agent".to_string(), serde_json::to_value(risk_analysis)?);

    // Return partial state update
    let mut result = PartialAgentStateUpdate::new();
    result = result.with_messages(vec![message]);
    result = result.with_data(updated_data);

    return Ok(result);  
  }

}