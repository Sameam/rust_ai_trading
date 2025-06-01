use std::collections::HashMap; 
use serde::{Serialize, Deserialize};
use serde_json::{Value};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Price {
  pub open : f64, 
  pub close : f64,
  pub high: f64, 
  pub low: f64, 
  pub volume : i64,
  pub time: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceResponse {
  pub ticker: String, 
  pub prices: Vec<Price>, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialMetrics {
  pub ticker: String,
  pub report_period: String, // Consider chrono::NaiveDate or similar
  pub period: String,
  pub currency: String,
  pub market_cap: Option<f64>,
  pub enterprise_value: Option<f64>,
  pub price_to_earnings_ratio: Option<f64>,
  pub price_to_book_ratio: Option<f64>,
  pub price_to_sales_ratio: Option<f64>,
  pub enterprise_value_to_ebitda_ratio: Option<f64>,
  pub enterprise_value_to_revenue_ratio: Option<f64>,
  pub free_cash_flow_yield: Option<f64>,
  pub peg_ratio: Option<f64>,
  pub gross_margin: Option<f64>,
  pub operating_margin: Option<f64>,
  pub net_margin: Option<f64>,
  pub return_on_equity: Option<f64>,
  pub return_on_assets: Option<f64>,
  pub return_on_invested_capital: Option<f64>,
  pub asset_turnover: Option<f64>,
  pub inventory_turnover: Option<f64>,
  pub receivables_turnover: Option<f64>,
  pub days_sales_outstanding: Option<f64>,
  pub operating_cycle: Option<f64>,
  pub working_capital_turnover: Option<f64>,
  pub current_ratio: Option<f64>,
  pub quick_ratio: Option<f64>,
  pub cash_ratio: Option<f64>,
  pub operating_cash_flow_ratio: Option<f64>,
  pub debt_to_equity: Option<f64>,
  pub debt_to_assets: Option<f64>,
  pub interest_coverage: Option<f64>,
  pub revenue_growth: Option<f64>,
  pub earnings_growth: Option<f64>,
  pub book_value_growth: Option<f64>,
  pub earnings_per_share_growth: Option<f64>,
  pub free_cash_flow_growth: Option<f64>,
  pub operating_income_growth: Option<f64>,
  pub ebitda_growth: Option<f64>,
  pub payout_ratio: Option<f64>,
  pub earnings_per_share: Option<f64>,
  pub book_value_per_share: Option<f64>,
  pub free_cash_flow_per_share: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialMetricsResponse {
  pub financial_metrics: Vec<FinancialMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItem {
  pub ticker: String,
  pub report_period: String, // Consider chrono::NaiveDate or similar
  pub period: String,
  pub currency: String,

  #[serde(flatten)]
  pub extra:         HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItemResponse {
  pub search_results: Vec<LineItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsiderTrade {
  pub ticker: String,
  pub issuer: Option<String>,
  pub name: Option<String>,
  pub title: Option<String>,
  pub is_board_director: Option<bool>,
  pub transaction_date: Option<String>, // Consider chrono::NaiveDate
  pub transaction_shares: Option<f64>,
  pub transaction_price_per_share: Option<f64>,
  pub transaction_value: Option<f64>,
  pub shares_owned_before_transaction: Option<f64>,
  pub shares_owned_after_transaction: Option<f64>,
  pub security_title: Option<String>,
  pub filing_date: Option<String> // Consider chrono::NaiveDate
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsiderTradeResponse {
  pub insider_trades: Vec<InsiderTrade>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyNews {
  pub ticker: String,
  pub title: String,
  pub author: String,
  pub source: String,
  pub date: String, // Consider chrono::DateTime<chrono::Utc> or NaiveDate
  pub url: String,
  pub sentiment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyNewsResponse {
  pub news: Vec<CompanyNews>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyFacts {
  pub ticker: String,
  pub name: String,
  pub cik: Option<String>,
  pub industry: Option<String>,
  pub sector: Option<String>,
  pub category: Option<String>,
  pub exchange: Option<String>,
  pub is_active: Option<bool>,
  pub listing_date: Option<String>, // Consider chrono::NaiveDate
  pub location: Option<String>,
  pub market_cap: Option<f64>,
  pub number_of_employees: Option<i64>,
  pub sec_filings_url: Option<String>,
  pub sic_code: Option<String>,
  pub sic_industry: Option<String>,
  pub sic_sector: Option<String>,
  pub website_url: Option<String>,
  pub weighted_average_shares: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyFactsResponse {
  pub company_facts: CompanyFacts,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
  #[serde(default)] // Pydantic default: 0.0
  pub cash: f64,
  #[serde(default)] // Pydantic default: 0
  pub shares: i64,
  pub ticker: String,
}

impl Default for Position {
  fn default() -> Self {
    Position {
      cash: 0.0,
      shares: 0,
      ticker: String::new(), // Default ticker to empty; usually provided on creation.
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
  #[serde(default)] // Pydantic default: empty dict
  pub positions: HashMap<String, Position>, // ticker -> Position mapping
  #[serde(default)] // Pydantic default: 0.0
  pub total_cash: f64,
}


impl Default for Portfolio {
  fn default() -> Self {
    Portfolio {
      positions: HashMap::new(),
      total_cash: 0.0,
    }
  }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStateMetaData {
  show_reasoning: bool,
}
