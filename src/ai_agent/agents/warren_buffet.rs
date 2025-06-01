use anyhow::{Error, Context, anyhow};
use serde_json::Value;
use std::collections::HashMap; 
use serde::{Serialize, Deserialize, Deserializer};
use std::result::Result::Err;
use std::str::FromStr;
use std::future::Future;
use std::pin::Pin;

use crate::ai_agent::graph::state::{AgentState, show_agent_reasoning, PartialAgentStateUpdate}; 
use crate::ai_agent::llm::models::get_model;
use crate::ai_agent::tools::api::API;
use crate::ai_agent::llm::model_provider::{ChatMessage, LLMModelConfig};
use crate::ai_agent::data::models::{FinancialMetrics,LineItem, };
use crate::ai_agent::llm::model_provider::{ModelProvider};
use crate::app::config::Config;


#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Signal {
  Bullish,
  Bearish,
  Neutral
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarrenBuffetSignal {
  #[serde(deserialize_with = "deserialize_signal")]
  signal : Signal, 
  confidence: f64, 
  reasoning: String
}

impl Signal {
  pub fn as_str(&self) -> &'static str {
    match self {
      Signal::Bearish => "bearish",
      Signal::Bullish => "bullish",
      Signal::Neutral => "neutral"
    }
  }
}

impl FromStr for Signal {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.trim().to_lowercase().as_str() {
      "bearish" => Ok(Signal::Bearish),  // Lowercase to match the lowercase input
      "bullish" => Ok(Signal::Bullish),    // Lowercase to match the lowercase input
      "neutral" => Ok(Signal::Neutral),        // Lowercase to match the lowercase input
      _ => Err(format!("Unknown model provider: {}", s)),
    }
  }
}

fn deserialize_signal<'de, D>(deserializer: D) -> Result<Signal, D::Error> where D: Deserializer<'de>,
{
    // First, deserialize as a String:
    let s = String::deserialize(deserializer)?;
    // Then parse it via your FromStr impl:
    Signal::from_str(&s).map_err(serde::de::Error::custom)
}

impl std::fmt::Display for Signal {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.as_str())
  }
}


impl WarrenBuffetSignal {
  pub fn new() -> Self {
    WarrenBuffetSignal { signal: Signal::Neutral, confidence: 0.0, reasoning: String::new() }
  }

  pub fn static_warren_buffet_agent(state: AgentState, config: Config) -> Pin<Box<dyn Future<Output = Result<PartialAgentStateUpdate, Error>> + Send>> {
    Box::pin(async move {
      let signal = WarrenBuffetSignal::new();
      signal.warren_buffet_agent(state, config).await
    })
  }

  pub async fn warren_buffet_agent(&self,state: AgentState, config: Config) -> Result<PartialAgentStateUpdate, Error> {

    let api_client : API = API::new(config); 
    let data : HashMap<String, Value> = state.data;
    let end_date: &str = match data.get("end_date").and_then(Value::as_str) {
      Some (value) => value,
      _ => {
        log::error!("Invalid end_date format for market cap");
        return Ok(PartialAgentStateUpdate::new());
      }
    };
    let tickers: Vec<String> = match data.get("tickers").and_then(Value::as_array) {
      Some(arr) if !arr.is_empty() => {
        arr.iter().filter_map(Value::as_str).map(String::from).collect()
      }
      _ => {
        log::error!("Invalid end_date format for market cap");
        return Ok(PartialAgentStateUpdate::new());
      }
    };


    let mut analysis_data: HashMap<String, HashMap<String, Value>> = HashMap::new();
    let mut buffet_analysis: HashMap<String, HashMap<String, Value>> = HashMap::new();

    if tickers.is_empty() {
      log::warn!("[Warren Buffett Agent] No tickers provided. Exiting.");
      return Ok(PartialAgentStateUpdate::new()); // Return empty update
    }

    for ticker in tickers {
      log::info!("Warren buffet agent {} fetching financial metrics", ticker); 

      let ticker: &str = ticker.as_str(); 

      let metrics: Vec<FinancialMetrics> = api_client.get_financial_metrics(ticker, end_date, Some("ttm"), Some(5)).await?;

      log::info!("Warren buffet agent {} gathering financial line items", ticker); 

      let line_items: Vec<String> = vec!["capital_expenditure", "depreciation_and_amortization","net_income",
                                                "outstanding_shares",
                                                "total_assets",
                                                "total_liabilities",
                                                "dividends_and_other_cash_distributions",
                                                "issuance_or_purchase_of_equity_shares",].into_iter().map(String::from).collect();

      let financial_line_items: Vec<LineItem> = api_client.search_line_items(ticker, line_items, end_date, Some("ttm"), Some(5)).await?;

      log::info!("Warren buffet agent {} Getting market cap", ticker);

      let market_cap: Option<f64> = api_client.get_market_cap(ticker, &end_date).await.with_context(|| format!("Failed to get market cap for {}", ticker))?;

      log::info!("warren_buffett_agent {} Analyzing fundamental", ticker); 

      let fundamental_analysis: HashMap<String, Value> = self.analyze_fundamental(&metrics)?;

      log::info!("warren_buffett_agent {} Analyzing consistency", ticker); 

      let consistency_analysis: HashMap<String, Value> = self.analyze_consistency(&financial_line_items)?;


      log::info!("warren_buffett_agent {} Analyzing moat", ticker); 
      let moat_analysis = self.analyze_moat(&metrics)?;

      log::info!("warren_buffett_agent {} Analyzing management quality", ticker);
      let mgmt_analysis = self.analyze_management_quality(&financial_line_items)?;

      log::info!("warren_buffett_agent {} Calculating intrinsic value", ticker);
      let intrinsic_value_analysis = self.calculate_intrinsic_value(&financial_line_items)?;

      // Calculate total score
      // Calculate total score
      let fundamental_score: i64 = fundamental_analysis.get("score").and_then(Value::as_i64).unwrap_or(0);
      let consistency_score: i64 = consistency_analysis.get("score").and_then(Value::as_i64).unwrap_or(0);
      let moat_score_val: i64 = moat_analysis.get("score").and_then(Value::as_i64).unwrap_or(0);
      let mgmt_score_val: i64 = mgmt_analysis.get("score").and_then(Value::as_i64).unwrap_or(0);
      let total_score: i64 = fundamental_score + consistency_score + moat_score_val + mgmt_score_val;

      let moat_max_score: i64 = moat_analysis.get("max_score").and_then(Value::as_i64).unwrap_or(3);
      let mgmt_max_score:i64 = mgmt_analysis.get("max_score").and_then(Value::as_i64).unwrap_or(2);

      let max_possible_score: i64 = 7 + 3 + moat_max_score + mgmt_max_score;

      let intrinsic_value = intrinsic_value_analysis.get("intrinsic_value").and_then(Value::as_f64);
      let margin_of_safety = match (intrinsic_value, market_cap) {
        (Some(iv), Some(mc)) if mc.abs() > 1e-6 => Some((iv - mc) / mc),
        _ => None,
      };

      let bullish_threshold: i64 = (0.7 * max_possible_score as f64) as i64;
      let bearish_threshold: i64 = (0.3 * max_possible_score as f64) as i64;

      let signal: Signal = if total_score >= bullish_threshold && margin_of_safety.map_or(false, |mos| mos >= 0.3) {
        Signal::Bullish
      } else if total_score <= bearish_threshold || margin_of_safety.map_or(false, |mos| mos < -0.3) {
        Signal::Bearish
      } else {
        Signal::Neutral
      };

      let mut result_data : HashMap<String, Value> = HashMap::new();

      result_data.insert("signal".to_string(), Value::from(signal.to_string()));
      result_data.insert("score".to_string(), Value::from(total_score));
      result_data.insert("max_score".to_string(), Value::from(max_possible_score)); 
      result_data.insert("fundamental_analysis".to_string(), serde_json::to_value(fundamental_analysis)?);
      result_data.insert("consistency_analysis".to_string(), serde_json::to_value(consistency_analysis)?); 
      result_data.insert("moat_analysis".to_string(), serde_json::to_value(moat_analysis)?); 
      result_data.insert("management_analysis".to_string(), serde_json::to_value(mgmt_analysis)?); 
      result_data.insert("intrinsic_value_analysis".to_string(), serde_json::to_value(intrinsic_value_analysis)?); 

      if let Some(mc) = market_cap { result_data.insert("market_cap".to_string(), Value::from(mc));} 
      if let Some(ms) = margin_of_safety { result_data.insert("margin_of_safety".to_string(), Value::from(ms));}

      analysis_data.insert(ticker.to_string(), result_data); 

      let ticker_data = analysis_data.get(&ticker.to_string()).expect("just inserted this key");    // Option<&HashMap<String,Value>>

      log::info!("[Warren Buffett Agent] ({}) Generating final signal via LLM...", ticker);

      let model_name: &str= if let Some(model_name) = state.metadata.get("model_name").and_then(Value::as_str) {
        model_name
      }else {
        log::error!("Metadata missing a model_name key");
        return Ok(PartialAgentStateUpdate::new());
      };
      let model_provider : &str =  if let Some(model_provider) = state.metadata.get("model_provider").and_then(Value::as_str) {
        model_provider
      } else {
        log::error!("Metadata missing a model_provider key");
        return Ok(PartialAgentStateUpdate::new());
      };

      let buffet_output = self.generate_buffet_output(ticker, ticker_data, model_name, model_provider).await?;

      let mut final_buffer : HashMap<String, Value> = HashMap::new(); 

      final_buffer.insert("signal".to_string(), Value::from(buffet_output.signal.to_string()));

      final_buffer.insert("confidence".to_string(), Value::from(buffet_output.confidence.to_string()));

      final_buffer.insert("reasoning".to_string(), Value::from(buffet_output.reasoning.to_string()));

      buffet_analysis.insert(ticker.to_string(), final_buffer); 
    }

    let message_content_string = serde_json::to_string(&buffet_analysis).context("Failed to serialize overall Buffett signal results to string for message")?;
    
    let agent_message = ChatMessage {
      role: "assistant".to_string(), // Or a custom role like "warren_buffett_agent_signal"
      content: message_content_string,
      // name: Some("warren_buffett_agent".to_string()), // If your ChatMessage struct supports a name
    };

    let show_reasoning = state.metadata.get("show_reasoning").and_then(Value::as_bool).unwrap_or(false);

    if show_reasoning {
      // show_agent_reasoning expects a &T where T: Serialize.
      // overall_buffett_signal_results is HashMap<String, WarrenBuffettSignal> which is Serialize.
      show_agent_reasoning(&agent_message.content, "Warren Buffett Agent");
    }

    let mut updated_data_map = HashMap::new();
    let final_buffett_analysis_value = serde_json::to_value(buffet_analysis).context("Failed to serialize overall Buffett signals to JSON Value for state update")?;

    // Create the nested structure: data -> analyst_signals -> warren_buffett_agent
    let mut analyst_signals_sub_map = HashMap::new();
    analyst_signals_sub_map.insert("warren_buffett_agent".to_string(), final_buffett_analysis_value);
    
    updated_data_map.insert("analyst_signals".to_string(), Value::Object(analyst_signals_sub_map.into_iter().collect()));

    log::info!("[Warren Buffett Agent] Analysis complete. Returning state update.");
    return Ok(PartialAgentStateUpdate {
      messages: Some(vec![agent_message]),
      data: Some(updated_data_map), // This will be merged into the main AgentState.data
      metadata: None, // No metadata changes made by this agent
    });
  }

  pub fn analyze_fundamental(&self, metrics: &[FinancialMetrics]) -> Result<HashMap<String, Value>, Error> {
    if metrics.is_empty() {
      let result : HashMap<String, Value> = HashMap::from([
        ("score".to_string(), Value::from(0.0)),
        ("details".to_string(), Value::from("Insufficient fundamental data"))]);
      return Ok(result);
    }

    let latest_metrics: FinancialMetrics = metrics[0].clone();
    let mut score: i64 = 0;
    let mut reasoning : Vec<String> = Vec::new();

    if let Some(roe) = latest_metrics.return_on_equity {
      if roe > 0.15 {
        score += 2;
        reasoning.push(format!("Strong ROE of {:.1}%", roe * 100.0));
      }
      else {
        reasoning.push(format!("Weak ROE of {:.1}%", roe * 100.0));
      }
    }
    else {
      reasoning.push("ROE data not available".to_string());
    }

    if let Some(de) = latest_metrics.debt_to_equity {
      if de < 0.5 {
        score += 2;
        reasoning.push(format!("Conservsative debt-to-equity ratio of {:.1}", de));
      }
      else {
        reasoning.push(format!("High debt-to-equity ratio of {:.1}", de));
      }
    }
    else {
      reasoning.push("Debt-to-equity data not available".to_string());
    }

    if let Some(op) = latest_metrics.operating_margin {
      if op > 0.15 {
        score += 2;
        reasoning.push(format!("Strong operating margin of {:.1}%", op * 100.0));
      }
      else {
        reasoning.push(format!("Weak operating margin of {:.1}%", op * 100.0));
      }
    }
    else {
      reasoning.push("Operating margin data not available".to_string());
    }

    if let Some(cr) = latest_metrics.current_ratio {
      if cr > 1.5 {
        score += 2;
        reasoning.push(format!("Good Liquidity with current ratio of {:.1}", cr));
      }
      else {
        reasoning.push(format!("Weak Liquidity with current ratio of {:.1}", cr));
      }
    }
    else {
      reasoning.push("Current ratio data not available".to_string());
    }

    let metrics_value: Value = serde_json::to_value(&latest_metrics)?;

    let mut result: HashMap<String, Value> = HashMap::new(); 
    result.insert("score".to_string(), Value::from(score)); 
    result.insert("reasoning".to_string(), Value::from(reasoning.join(" "))); 
    result.insert("metrics".to_string(), metrics_value);

    return Ok(result);

  }

  pub fn analyze_consistency(&self, financial_line_items: &[LineItem]) -> Result<HashMap<String, Value>, Error> {
    // Analyze earning consistency and growth 

    if financial_line_items.len() < 4 {
      let result = HashMap::from([
        ("score".to_string(), Value::from(0)), 
        ("details".to_string(), Value::from("Insufficient historical data"))
      ]);
      return Ok(result);
    }

    let mut score: i64 = 0;
    let mut reasoning : Vec<String> = Vec::new(); 

    let earning_values : Vec<f64> = financial_line_items.iter().filter_map(|item| {item.extra.get("net_income").and_then(Value::as_f64)}).collect();    // Option<&Value>

    if earning_values.len() >= 4 {
      let earning_growth  = earning_values.windows(2).all(|w| w[0] > w[1]);

      if earning_growth {
        score += 3; 
        reasoning.push("Consistent earnings growth over the past period.".to_string());
      }
      else {
        reasoning.push("Inconsistent earnings growths pattern".to_string());
      }

      if earning_values.len() >= 2 {
        let latest_earning = earning_values.first().unwrap_or(&0.0);
        let oldest_earning_in_window = earning_values.last().unwrap_or(&0.0); 

        if oldest_earning_in_window.abs() > 1e-6 {
          let growth_rate = (latest_earning - oldest_earning_in_window) / oldest_earning_in_window.abs();
          reasoning.push(format!("Total earnings growth of {:.1}% over considered {} periods", growth_rate * 100.0, earning_values.len()));
        }
      }

    }

    else {
      reasoning.push("Insufficient earnings data for trend analysis".to_string());
    }

    let mut final_response: HashMap<String, Value> = HashMap::new(); 
    final_response.insert("score".to_owned(), Value::from(score));
    final_response.insert("details".to_string(), Value::from(reasoning.join("; ")));

    return Ok(final_response);

  }

  pub fn analyze_moat(&self, metrics: &[FinancialMetrics]) -> Result<HashMap<String, Value>, Error> {
    /*Evaluate whether the company likely has a durable competitive advantage (moat).
    For simplicity, we look at stability of ROE/operating margins over multiple periods
    or high margin over the last few years. Higher stability => higher moat score. */

    if metrics.len() < 3 {
      let result = HashMap::from([
        ("score".to_string(), Value::from(0)), 
        ("max_score".to_string(), Value::from(3)),
        ("details".to_string(), Value::from("Insufficient data for moat analysis"))
      ]);
      return Ok(result);
    }

    let mut reasoning : Vec<String> = Vec::new();
    let mut moat_score : i64 = 0; 
    let historical_roes: Vec<f64> = metrics.iter().filter_map(|m| m.return_on_equity).collect();
    let historical_margins: Vec<f64> = metrics.iter().filter_map(|m| m.operating_margin).collect();

    if historical_roes.len() >= 3 && historical_roes.iter().all(|&r| r > 0.15) {
      moat_score += 1; 
      reasoning.push("Stable ROE above 15% across periods (suggests moat)".to_string());
    }
    else {
      reasoning.push("ROE not consistently above 15%".to_string());
    }

    if historical_margins.len() >= 3 && historical_margins.iter().all(|&r| r > 0.15)  {
      moat_score += 1;
      reasoning.push("Stable operating margin above 15% (moat score indicator)".to_string());
    }
    else {
      reasoning.push("Operating margin not consistently above 15%".to_string());
    }

    if moat_score == 2 {
      moat_score += 1; 
      reasoning.push("Both ROE and margin stability indicate a solid moat".to_string());
    }

    let mut final_result : HashMap<String, Value> = HashMap::new();

    final_result.insert("score".to_string(), Value::from(moat_score)); 
    final_result.insert("max_score".to_string(), Value::from(3));
    final_result.insert("details".to_string(), Value::from(reasoning)); 

    return Ok(final_result);

  }


  pub fn analyze_management_quality(&self, financial_line_items: &[LineItem]) -> Result<HashMap<String, Value>, Error> {
    /* 
    Checks for share dilution or consistent buybacks, and some dividend track record.
    A simplified approach:
      - if there's net share repurchase or stable share count, it suggests management
        might be shareholder-friendly.
      - if there's a big new issuance, it might be a negative sign (dilution).
     */

    if financial_line_items.is_empty() {
      let result : HashMap<String, Value> = HashMap::from([
        ("score".to_string(),Value::from(0)),
        ("max_score".to_string(), Value::from(2)), 
        ("details".to_string(), Value::from("Insufficient data for management analysis"))
      ]);

      return Ok(result);
    }


    let mut reasoning :Vec<String> = Vec::new(); 
    let mut mgmt_score : i64 = 0; 
    let latest = &financial_line_items[0];

    if let Some(issuance_purchase) = latest.extra.get("issuance_or_purchase_of_equity_shares").and_then(Value::as_f64) {
      if issuance_purchase < 0.0 {
        mgmt_score += 1; 
        reasoning.push("Company has been repurchasing shares (shareholder-friendly)".to_string());
      }
      else if issuance_purchase > 0.0 {
        reasoning.push("Recent common stock issuance (potential dilution)".to_string());
      }
      else {
        reasoning.push("No significant new stock issuance detected".to_string());
      }
    }
    else {
      reasoning.push("Data on stock issuance/repurchase not available".to_string());
    }

    if let Some(dividends) = latest.extra.get("dividends_and_other_cash_distributions").and_then(Value::as_f64) {
      if dividends < 0.0 {
        mgmt_score += 1; 
        reasoning.push("Company has a track record of paying dividends".to_string());
      }
      else {
        reasoning.push("No or minimal dividend paids".to_string()); 
      }
    }
    else {
      reasoning.push("Dividend payment data not available".to_string());
    }

    let mut final_result : HashMap<String, Value> = HashMap::new(); 
    final_result.insert("score".to_string(), Value::from(mgmt_score)); 
    final_result.insert("max_score".to_string(), Value::from(2)); 
    final_result.insert("details".to_string(), Value::from(reasoning.join(" ,"))); 

    return Ok(final_result);

  }

  pub fn calculate_owner_earnings(&self, financial_line_items: &[LineItem]) -> Result<HashMap<String, Value>, Error> {
    /* Calculate owner earnings (Buffett's preferred measure of true earnings power).
    Owner Earnings = Net Income + Depreciation - Maintenance CapEx 
    */

    if financial_line_items.is_empty() {
      let _result : HashMap<String, Value> = HashMap::from([
        ("owner_earnings".to_string(), Value::Null), ("details".to_string(), Value::from(vec![Value::from("Insufficent data for owner earnings calculation")]))
      ]);
    }

    let latest = &financial_line_items[0];

    let mut details = Vec::new(); 

    match (latest.extra.get("net_income").and_then(Value::as_f64), 
          latest.extra.get("depreciation_and_amortization").and_then(Value::as_f64), 
          latest.extra.get("capital_expenditure").and_then(Value::as_f64)) {
      (Some(net_income), Some(depreciation), Some(capex))  => {
        let maintenance_capex: f64 = capex * 0.75;
        let owner_earnings: f64 = net_income + depreciation - maintenance_capex;

        let mut result : HashMap<String, Value> = HashMap::new(); 
        let mut components : HashMap<String, Value> = HashMap::new(); 

        details.push("Owner earnings calculated successfully".to_string());

        components.insert("net_income".to_string(), Value::from(net_income)); 
        components.insert("depreciation".to_string(), Value::from(depreciation)); 
        components.insert("maintenance_capex".to_string(), Value::from(maintenance_capex));

        result.insert("owner_earnings".to_string(), Value::from(owner_earnings)); 
        result.insert("components".to_string(), serde_json::to_value(components)?); 
        result.insert("details".to_string(), Value::from(details.into_iter().map(Value::from).collect::<Vec<_>>())); 
        return Ok(result); 
      }
      _ => {
        details.push("Missing components for owner earnings calculation".to_string());
        return Ok(HashMap::from([
          ("owner_earnings".to_string(), Value::Null),
          ("details".to_string(), Value::from(details.into_iter().map(Value::from).collect::<Vec<_>>())),
        ]));
      }
    }

  }

  pub fn calculate_intrinsic_value(&self, financial_line_items: &[LineItem]) -> Result<HashMap<String, Value>, Error> {
    if financial_line_items.is_empty() {
      let result : HashMap<String, Value> = HashMap::from([
        ("intrinsic_value".to_string(), Value::Null), ("details".to_string(), Value::from(vec![Value::from("Insufficient data for valuation")]))
      ]);
      return Ok(result);
    }


    let earning_data: HashMap<String, Value> = self.calculate_owner_earnings(&financial_line_items)?; 

    let owner_earnings: f64 = match earning_data.get("owner_earnings").and_then(Value::as_f64) {
      Some(owner_earning) => owner_earning, // bind the f64 here
      None => {
        let details = earning_data.get("details").cloned().unwrap_or(Value::from("Missing earning data".to_string()));
        return Ok(HashMap::from([("intrinsic_value".to_string(), Value::Null),("details".to_string(), details),]));
      }
    };

    let shares_outstanding = financial_line_items[0].extra.get("outstanding_shares").and_then(Value::as_f64);

    if shares_outstanding.is_none() {
      return Ok(HashMap::from([
        ("intrinsic_value".to_string(), Value::Null),
        ("details".to_string(), Value::from(vec![Value::from("Missing shares outstanding data")])),
      ]));
    }

    let growth_rate : f64 = 0.05; 
    let discount_rate : f64 = 0.09; 
    let terminal_multiple : i64 = 12; 
    let projection_years : i32 = 10;

    let mut future_values : f64 = 0.0; 

    for year in 1..=projection_years {
      let future_earnings : f64 = owner_earnings * (1.0 + growth_rate) .powi(year);
      let present_value : f64 = future_earnings / (1.0 + growth_rate).powi(year); 
      future_values += present_value; 
    }

    let terminal_owner_earnings_proj : f64 = owner_earnings * (1.0 + growth_rate).powi(projection_years);
    let terminal_value_at_proj_end = terminal_owner_earnings_proj * terminal_multiple as f64;

    let terminal_value : f64 = terminal_value_at_proj_end / (1.0 + discount_rate).powi(projection_years); 
    let intrinsic_value : f64 = future_values + terminal_value;

    let mut assumption : HashMap<String, Value> = HashMap::new();
    let mut result : HashMap<String, Value> = HashMap::new();

    assumption.insert("growth_rate".to_string(), Value::from(growth_rate)); 
    assumption.insert("discount_rate".to_string(), Value::from(discount_rate)); 
    assumption.insert("terminal_multiple".to_string(), Value::from(terminal_multiple)); 
    assumption.insert("projection_years".to_string(), Value::from(projection_years)); 


    result.insert("intrinsic_value".to_string(), Value::from(intrinsic_value)); 
    result.insert("owner_earnings".to_string(), Value::from(owner_earnings));
    result.insert("assumptions".to_string(), serde_json::to_value(assumption)?);
    result.insert("details".to_string(), Value::from(vec![Value::from("Intrinsic value calculated using DCF model with owner earnings")]));

    return Ok(result);
  }

  pub async fn generate_buffet_output(&self, ticker: &str, analysis_data: &HashMap<String, Value>, model_name: &str, model_provider: &str) -> Result<WarrenBuffetSignal, Error> {

    let analysis_data_json = serde_json::to_string_pretty(analysis_data).context("Failed to serialize analysis data for LLM prompt")?;

    let system_prompt : &str = r#"You are a Warren Buffett AI agent. Decide on investment signals based on Warren Buffett's principles:
                                  - Circle of Competence: Only invest in businesses you understand
                                  - Margin of Safety (> 30%): Buy at a significant discount to intrinsic value
                                  - Economic Moat: Look for durable competitive advantages
                                  - Quality Management: Seek conservative, shareholder-oriented teams
                                  - Financial Strength: Favor low debt, strong returns on equity
                                  - Long-term Horizon: Invest in businesses, not just stocks
                                  - Sell only if fundamentals deteriorate or valuation far exceeds intrinsic value

                                  When providing your reasoning, be thorough and specific by:
                                  1. Explaining the key factors that influenced your decision the most (both positive and negative)
                                  2. Highlighting how the company aligns with or violates specific Buffett principles
                                  3. Providing quantitative evidence where relevant (e.g., specific margins, ROE values, debt levels)
                                  4. Concluding with a Buffett-style assessment of the investment opportunity
                                  5. Using Warren Buffett's voice and conversational style in your explanation

                                  For example, if bullish: "I'm particularly impressed with [specific strength], reminiscent of our early investment in See's Candies where we saw [similar attribute]..."
                                  For example, if bearish: "The declining returns on capital remind me of the textile operations at Berkshire that we eventually exited because..."

                                  Follow these guidelines strictly."#;

    let human_prompt : String = format!(r#"Based on the following data, create the investment signal as Warren Buffett would:
                              Analysis Data for {}:
                              {}

                              Return the trading signal in the following JSON format exactly without any explanation:
                              {{
                                "signal": "bullish" | "bearish" | "neutral",
                                "confidence": float between 0 and 100,
                                "reasoning": "string"
                              }}"#, ticker, analysis_data_json);

    let user_prompt: String = human_prompt;

    let messages = vec![
      ChatMessage{ role: "system".to_string(), content: system_prompt.to_string()}, 
      ChatMessage{ role: "user".to_string(), content: user_prompt}
    ]; 


    let provider = ModelProvider::from_str(model_provider).map_err(|_| anyhow!("Unknown model provider: {}",model_provider))?;

    let config_for_call : LLMModelConfig = LLMModelConfig { 
      provider: provider, 
      model_name: model_name.to_string(), 
      api_key:Some("".to_string()) , 
      base_url: Some("".to_string()), 
      temperature: Some(0.5), 
      max_tokens: Some(1024), 
      top_p: Some(0.5)
    };

    let client = get_model(&config_for_call)?;

    log::info!("[Warren Buffett Agent] ({}) Calling LLM for Buffett analysis...", ticker);

    let response = client.chat(messages, &config_for_call).await?; 

    log::debug!("[Warren Buffett Agent] ({}) LLM raw response: {}", ticker, response.content);
    
    match serde_json::from_str::<WarrenBuffetSignal>(&response.content) {
      Ok(signal) => return Ok(signal),
      Err(err) => {
        log::error!("[Warren Buffett Agent] ({}) Failed to parse LLM JSON response into WarrenBuffettSignal: {}. Raw response: '{}'",ticker,err,response.content);
        Ok(WarrenBuffetSignal {
          signal: Signal::Neutral,
          confidence: 0.0,
          reasoning: format!("Error in LLM analysis or response parsing for ticker {}: {}. Defaulting to neutral.", ticker, err),
        })
      }
    }

  }

}