use crate::ai_agent::{graph::state::{show_agent_reasoning, AgentState, PartialAgentStateUpdate}, llm::model_provider::{ChatMessage, LLMModelConfig}};
use crate::ai_agent::llm::model_provider::{ModelProvider};
use crate::ai_agent::llm::models::get_model;
use crate::app::config::Config;

use std::{collections::HashMap, result::Result}; 
use std::str::FromStr;
use anyhow::{Error, anyhow};
use serde::{Deserialize, Serialize, Deserializer};
use serde_json::Value;
use std::{pin::Pin, future::Future};

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum Action {
  Buy, Sell, Short, Cover, Hold
}

impl Action {

  pub fn _as_str(&self) -> &'static str {
    match self {
      &Action::Buy => "buy",
      &Action::Sell => "sell",
      &Action::Short => "short",
      &Action::Cover => "cover",
      &Action::Hold => "hold",
    }
  }
}

impl FromStr for Action {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.trim().to_lowercase().as_str() {
      "buy" => Ok(Action::Buy),  // Lowercase to match the lowercase input
      "sell" => Ok(Action::Sell),    // Lowercase to match the lowercase input
      "short" => Ok(Action::Short),        // Lowercase to match the lowercase input
      "cover" => Ok(Action::Cover),        // Lowercase to match the lowercase 
      "hold" => Ok(Action::Hold),        // Lowercase to match the 
      _ => Err(format!("Unknown model provider: {}", s)),
    }
  }
}

fn deserialize_signal<'de, D>(deserializer: D) -> Result<Action, D::Error> where D: Deserializer<'de>,
{
    // First, deserialize as a String:
    let s = String::deserialize(deserializer)?;
    // Then parse it via your FromStr impl:
    Action::from_str(&s).map_err(serde::de::Error::custom)
}


#[derive(Clone, Serialize, Deserialize)]
pub struct PortfolioDecision {
  #[serde(deserialize_with = "deserialize_signal")]
  action : Action,
  quantity: i64,
  confidence: f64,
  reasoning: String
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PortfolioManagerOutput {
  decisions : HashMap<String, PortfolioDecision>
}

pub struct PortfolioManagerAgent; 

impl PortfolioManagerAgent {
  pub fn new() -> Self {
    PortfolioManagerAgent{}
  }

  pub fn static_portfolio_management_agent(state: AgentState, config: Config) -> Pin<Box<dyn Future<Output = Result<PartialAgentStateUpdate, Error>> + Send>> {
    Box::pin(async move {
      let portfolio_management = PortfolioManagerAgent::new();
      portfolio_management.portfolio_management_agent(state, config).await
    })
  }

  pub async fn portfolio_management_agent(&self, state: AgentState, config: Config) -> Result<PartialAgentStateUpdate, Error> {

    let portfolio = match state.data.get("portfolio") {
      Some(portfolio) => portfolio, 
      _ => {
        log::error!("Cannot find portfolio inside state.data"); 
        return Ok(PartialAgentStateUpdate::new());
      }
    }; 

    let analyst_signals = match state.data.get("analyst_signals") {
      Some(analyst_signals) => analyst_signals, 
      _ => {
        log::error!("Cannot find analyst signals inside state.data"); 
        return Ok(PartialAgentStateUpdate::new());
      }
    };

    let tickers: Vec<String> = match state.data.get("tickers").and_then(Value::as_array) {
      Some(arr) if !arr.is_empty() => {
        arr.iter().filter_map(Value::as_str).map(String::from).collect()
      }
      _ => {
        log::error!("Invalid end_date format for market cap");
        return Ok(PartialAgentStateUpdate::new());
      }
    };

    let mut position_limits: HashMap<String, f64> = HashMap::new(); 
    let mut current_prices: HashMap<String, f64> = HashMap::new(); 
    let mut max_shares : HashMap<String, i64> = HashMap::new(); 
    let mut signals_by_ticker: HashMap<String, HashMap<String, Value>> = HashMap::new(); 

    for ticker in &tickers {

      log::info!("Portfolio manager: {} , processing analyst signals", ticker);

      let risk_data: &Value = analyst_signals.get("risk_management_agent").and_then(|agent| agent.as_object()).and_then(|agent_obj| agent_obj.get(ticker)).unwrap_or(&Value::Null);

      let position_limit: f64 = risk_data.get("remaining_position_limit").and_then(Value::as_f64).unwrap_or(0.0);

      position_limits.insert(ticker.clone(), position_limit);

      let current_price = risk_data.get("current_price").and_then(Value::as_f64).unwrap_or(0.0); 

      current_prices.insert(ticker.clone(), current_price); 

      let max_share = if current_price > 0.0 {
        (position_limit / current_price) as i64
      }
      else {
        0
      };

      max_shares.insert(ticker.clone(), max_share); 

      let mut ticker_signals: HashMap<String, Value> = HashMap::new();

      if let Some(signals_objs) = analyst_signals.as_object() {
        for (agent, signals) in signals_objs {
          if agent != "risk_management_agent" {
            if let Some(agent_obj) = signals.as_object() {
              if let Some(ticker_signal) = agent_obj.get(ticker) {
                let mut signal_data = HashMap::new(); 
                if let Some(signal) = ticker_signal.get("signal").and_then(Value::as_str) {
                  signal_data.insert("signal".to_string(), Value::String(signal.to_string()));
                }
                
                if let Some(confidence) = ticker_signal.get("confidence").and_then(Value::as_f64) {
                  signal_data.insert("confidence".to_string(), Value::from(confidence));
                }
                
                ticker_signals.insert(agent.clone(), Value::Object(signal_data.into_iter().collect()));
              }
            }
          }
        }
      }
      signals_by_ticker.insert(ticker.clone(), ticker_signals); 
    }

    log::info!("Portfolio_manager generating trading decision");

    let model_name: &str= if let Some(model_name) = state.metadata.get("model_name").and_then(Value::as_str) {
      model_name
    }else {
      log::error!("Metadata missing a model_name key");
      return Ok(PartialAgentStateUpdate::new());
    };
    let model_provider = if let Some(model_provider) = state.metadata.get("model_provider").and_then(Value::as_str) {
      model_provider
    }
    else {
      log::error!("Metadata missing a model_name key");
      return Ok(PartialAgentStateUpdate::new());
    };

    let result = self.generate_trading_decision(config, &tickers, &signals_by_ticker, &current_prices, &max_shares, portfolio, model_name, &model_provider).await?;

    let message_content = serde_json::to_string(&result.decisions)?;

    let message = ChatMessage {
      role: "assistant".to_string(),
      content: message_content.clone(),
    };

    if let Some(show_reasoning) = state.metadata.get("show_reasoning").and_then(Value::as_bool) {
      if show_reasoning {
        show_agent_reasoning(&message_content, "Portfolio Manager");
      }
    }

    let mut result = PartialAgentStateUpdate::new();
    result = result.with_messages(vec![message]);
    result = result.with_data(state.data.clone());

    return Ok(result);  

  }


  pub async fn generate_trading_decision(&self, config: Config, tickers: &[String], signals_by_ticker : &HashMap<String, HashMap<String, Value>>, 
                                  current_prices: &HashMap<String, f64>, max_shares: &HashMap<String, i64>, portfolio: &Value,
                                  model_name: &str, model_provider: &str) -> Result<PortfolioManagerOutput, Error> {

    let portfolio_cash: f64 = portfolio.get("cash").and_then(Value::as_f64).unwrap_or(0.0);
    let portfolio_position = portfolio.get("positions").cloned().unwrap_or_else(|| Value::Object(serde_json::Map::new()));
    let margin_requirement: f64 = portfolio.get("margin_requirement").and_then(Value::as_f64).unwrap_or(0.0); 
    let total_margin_used: f64 = portfolio.get("margin_used").and_then(Value::as_f64).unwrap_or(0.0); 

    let system_prompt = r#"You are a portfolio manager making final trading decisions based on multiple tickers.
                                        Trading Rules:
                                          - For long positions:
                                            * Only buy if you have available cash
                                            * Only sell if you currently hold long shares of that ticker
                                            * Sell quantity must be ≤ current long position shares
                                            * Buy quantity must be ≤ max_shares for that ticker

                                          - For short positions:
                                            * Only short if you have available margin (position value × margin requirement)
                                            * Only cover if you currently have short shares of that ticker
                                            * Cover quantity must be ≤ current short position shares
                                            * Short quantity must respect margin requirements

                                          - The max_shares values are pre-calculated to respect position limits
                                          - Consider both long and short opportunities based on signals
                                          - Maintain appropriate risk management with both long and short exposure

                                          Available Actions:
                                          - "buy": Open or add to long position
                                          - "sell": Close or reduce long position
                                          - "short": Open or add to short position
                                          - "cover": Close or reduce short position
                                          - "hold": No action

                                          Inputs:
                                          - signals_by_ticker: dictionary of ticker → signals
                                          - max_shares: maximum shares allowed per ticker
                                          - portfolio_cash: current cash in portfolio
                                          - portfolio_positions: current positions (both long and short)
                                          - current_prices: current prices for each ticker
                                          - margin_requirement: current margin requirement for short positions (e.g., 0.5 means 50%)
                                          - total_margin_used: total margin currently in use"#;

    let human_prompt = format!(r#"Based on the team's analysis, make your trading decisions for each ticker.
                                        Here are the signals by ticker:
                                        {}

                                        Current Prices:
                                        {}

                                        Maximum Shares Allowed For Purchases:
                                        {}

                                        Portfolio Cash: {:.2}
                                        Current Positions: {}
                                        Current Margin Requirement: {:.2}
                                        Total Margin Used: {:.2}

                                        Output strictly in JSON with the following structure without any explanation:
                                        {{
                                          "decisions": {{
                                            "TICKER1": {{
                                              "action": "buy/sell/short/cover/hold",
                                              "quantity": integer,
                                              "confidence": float between 0 and 100,
                                              "reasoning": "string"
                                            }},
                                            "TICKER2": {{
                                              ...
                                            }},
                                            ...
                                          }}
                                        }}
                                        "#, 
                                      serde_json::to_string_pretty(signals_by_ticker)?,serde_json::to_string_pretty(current_prices)?,
                                      serde_json::to_string_pretty(max_shares)?, portfolio_cash, serde_json::to_string_pretty(&portfolio_position)?,
                                      margin_requirement, total_margin_used);

    let messages = vec![
      ChatMessage {
        role: "system".to_string(), 
        content: system_prompt.to_string()
      }, 
      ChatMessage {
        role: "user".to_string(),
        content: human_prompt
      }
    ]; 

    let provider = ModelProvider::from_str(model_provider).map_err(|_| anyhow!("Unknown model provider: {}",model_provider))?;

    let config_for_call : LLMModelConfig = LLMModelConfig { 
      provider: provider, 
      model_name: model_name.to_string(), 
      api_key:Some(config.groq_api_key.to_string()) , 
      base_url: Some("".to_string()), 
      temperature: Some(0.5), 
      max_tokens: Some(1024), 
      top_p: Some(0.5)
    };

    let model = get_model(&config_for_call)?; 

    log::info!("Calling LLM for portfolio decisions...");
    let response = model.chat(messages, &config_for_call).await?;
    log::debug!("LLM response: {}", response.content);


    match serde_json::from_str::<PortfolioManagerOutput>(&response.content) {
      Ok(output) => Ok(output),
      Err(e) => {
        log::error!("Failed to parse LLM response: {}", e);
        
        // Create a default output
        let mut decisions = HashMap::new();
        for ticker in tickers {
          decisions.insert(ticker.clone(), PortfolioDecision {
            action: Action::Hold,
            quantity: 0,
            confidence: 0.0,
            reasoning: "Error in portfolio management, defaulting to hold".to_string(),
          });
        }
        
        Ok(PortfolioManagerOutput { decisions })
      }
    }
  }


}