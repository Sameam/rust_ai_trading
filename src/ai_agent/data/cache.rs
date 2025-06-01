use anyhow::{Error, Ok};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

pub struct Cache {
  price_cache: HashMap<String, Vec<HashMap<String, Value>>>,
  financial_metric_cache: HashMap<String, Vec<HashMap<String, Value>>>,
  line_items_cache: HashMap<String, Vec<HashMap<String, Value>>>,
  insider_trades_cache: HashMap<String, Vec<HashMap<String, Value>>>,
  company_news_cache: HashMap<String, Vec<HashMap<String, Value>>>,
}

static GLOBAL_CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();

impl Cache {
  pub fn new() -> Self {
    Cache {
      price_cache: HashMap::new(),
      financial_metric_cache: HashMap::new(),
      line_items_cache: HashMap::new(),
      insider_trades_cache: HashMap::new(),
      company_news_cache: HashMap::new(),
    }
  }

  fn merge_data(&self,existing: Vec<HashMap<String, Value>>, new_data: Vec<HashMap<String, Value>>,key_field: &str,) -> Result<Vec<HashMap<String, Value>>, Error> {
    let mut merged = existing.clone();

    for new_item in new_data {
      let key = new_item.get(key_field).ok_or_else(|| Error::msg(format!("Missing key field: {}", key_field)))?.to_string();

      if !merged.iter().any(|item| { 
        item.get(key_field).map_or(false, |v| v == &Value::String(key.clone()))}) {
        merged.push(new_item);
      }
    }

    Ok(merged)
  }

  pub fn get_prices(&self, ticker: &str) -> Result<Vec<HashMap<String, Value>>, Error> {
    let result = self.price_cache.get(ticker);
    match result {
      Some(result) =>  {return Ok(result.clone()) },
      None => {
        log::info!("Price does not match with ticker {}", ticker);
        return Ok(Vec::new());
      }
    }
  }

  pub fn set_prices(&mut self, ticker: &str, data: Vec<HashMap<String, Value>>) -> Result<(), Error> {
    let result = self.price_cache.get(ticker).cloned().unwrap_or_default();

    let merged_data = self.merge_data(result, data, "time")?;
    self.price_cache.insert(ticker.to_string(), merged_data);
    Ok(())

   
  }

  pub fn get_financial_metrics(&self, ticker: &str) -> Result<Vec<HashMap<String, Value>>, Error> {
    let result = self.financial_metric_cache.get(ticker);

    match result {
      Some(result) => return Ok(result.clone()), 
      None =>  {
        log::info!("Financial metrics does not match with ticker {}", ticker);
        return Ok(Vec::new());
      }
    }
  }

  pub fn set_financial_metrics(&mut self, ticker: &str, data: Vec<HashMap<String, Value>>) -> Result<(), Error> {
    let result = self.financial_metric_cache.get(ticker).cloned().unwrap_or_default();
    let merged_data = self.merge_data(result, data, "report_period")?;
    self.financial_metric_cache.insert(ticker.to_string(), merged_data);
    Ok(())
  }


  pub fn get_line_items(&self, ticker: &str) -> Result<Vec<HashMap<String, Value>>, Error> {
    match self.line_items_cache.get(ticker) {
      Some(items_vec_ref) => Ok(items_vec_ref.clone()),
      None => {
        log::info!("Line items not found in cache for ticker: {}", ticker);
        Ok(Vec::new())
      }
    }
  }

  pub fn set_line_items(&mut self, ticker: &str, data: Vec<HashMap<String, Value>>) -> Result<(), Error> {
    let existing_data_for_ticker = self.line_items_cache.get(ticker).cloned().unwrap_or_default();
    let merged_data = self.merge_data(existing_data_for_ticker, data, "report_period")?;
    self.line_items_cache.insert(ticker.to_string(), merged_data);
    Ok(())
  }

  pub fn get_insider_trades(&self, ticker: &str) -> Result<Vec<HashMap<String, Value>>, Error> {
    match self.insider_trades_cache.get(ticker) {
      Some(trades_vec_ref) => Ok(trades_vec_ref.clone()),
      None => {
        log::info!("Insider trades not found in cache for ticker: {}", ticker);
        Ok(Vec::new())
      }
    }
  }

  pub fn set_insider_trades(&mut self, ticker: &str, data: Vec<HashMap<String, Value>>) -> Result<(), Error> {
    let existing_data_for_ticker = self.insider_trades_cache.get(ticker).cloned().unwrap_or_default();
    let merged_data = self.merge_data(existing_data_for_ticker, data, "filing_date")?;
    self.insider_trades_cache.insert(ticker.to_string(), merged_data);
    Ok(())
  }

  pub fn get_company_news(&self, ticker: &str) -> Result<Vec<HashMap<String, Value>>, Error> {
    match self.company_news_cache.get(ticker) {
      Some(news_vec_ref) => Ok(news_vec_ref.clone()),
      None => {
        log::info!("Company news not found in cache for ticker: {}", ticker);
        Ok(Vec::new())
      }
    }
  }

  pub fn set_company_news(&mut self, ticker: &str, data: Vec<HashMap<String, Value>>) -> Result<(), Error> {
    let existing_data_for_ticker = self.company_news_cache.get(ticker).cloned().unwrap_or_default();
    let merged_data = self.merge_data(existing_data_for_ticker, data, "date")?;
    self.company_news_cache.insert(ticker.to_string(), merged_data);
    Ok(())
  }

}

pub fn get_cache() -> &'static Mutex<Cache> {
  GLOBAL_CACHE.get_or_init(|| {
      // This closure is executed only once by get_or_init
      log::info!("Global cache initialized."); // Optional: for logging
      Mutex::new(Cache::new()) // Create and wrap the new Cache instance
  })
}

