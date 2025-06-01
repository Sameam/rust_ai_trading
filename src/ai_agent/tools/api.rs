use crate::ai_agent::data::models::{
    CompanyFactsResponse, CompanyNews, CompanyNewsResponse, FinancialMetrics,
    FinancialMetricsResponse, InsiderTrade, InsiderTradeResponse, LineItem, LineItemResponse,
    Price, PriceResponse,
};
use crate::ai_agent::data::data::{FinancialHeaderData, LineItemBodyData};
use crate::ai_agent::data::cache::{self, Cache};
use crate::app::config::Config;


use reqwest::{Client, Error, Response};
use reqwest::header::{HeaderMap, HeaderValue};
use std::sync::Mutex;
use std::result::Result::{Ok, Err};
use std::option::Option;
use chrono::NaiveDate;
use std::collections::HashMap;
use serde::de::DeserializeOwned;
use serde_json::Value;
use polars::prelude::{Series, NamedFrom, DataFrame, TimeUnit, StringMethods, IntoSeries, SortMultipleOptions};
use std::env;



pub struct API {
  header_key : &'static str,
  config : Config
}

impl API {
  pub fn new(config: Config) -> Self {
    let header_key = "X-API-KEY";
    API {
      header_key, config
    }
  }

  pub async fn get_price(&self,ticker: &str,start_date: &str,end_date: &str,) -> Result<Vec<Price>, Error> {
    let cache : &'static Mutex<Cache> = cache::get_cache();

    {
      let cache_guard = cache.lock().unwrap();
      let result = cache_guard.get_prices(ticker);

      match result {
        Ok(data) if !data.is_empty() => {
          let prices : Vec<Price> = data.into_iter().filter_map( |h_map|{
            match serde_json::to_value(h_map) {
              Ok(json_value) => match serde_json::from_value(json_value) {
                Ok(price_struct) => Some(price_struct), 
                Err(e) => {
                  log::warn!("Failed to deserialize cached price item for {}: {}", ticker, e);
                  None
                }
              }, 
              Err(e) => {
                log::warn!("Failed to convert cached HashMap to Value for {}: {}", ticker, e);
                None
              }
            }
          }).collect();

          if !prices.is_empty() {
            log::info!("Returning prices for ticker {} from cache.", ticker);
            // TODO: Optionally filter 'prices' by start_date and end_date if cache stores more than requested.
            return Ok(prices);
          }
          else {
            log::info!("Cached data for {} was empty or failed deserialization.", ticker);
          }
        },
        Ok(_) => {
          log::info!("Cache miss (empty data) for prices (ticker: {}).", ticker);
        },
        Err(e) => {
          log::error!("Error accessing cache for prices (ticker: {}): {}. Proceeding to API call.",ticker,e);
        }
      }
    }

    log::info!("End date for get_price: {}", end_date);
    let url : String = format!("https://api.financialdatasets.ai/prices/?ticker={}&interval=day&interval_multiplier=1&start_date={}&end_date=2025-06-01", ticker, start_date);
    log::debug!("API URL: {}", url);
    let api_key: String = self.config.financial_datasets_api_key.to_string();
    log::debug!("Get price API key: {}", api_key);
    let headers: HeaderMap = FinancialHeaderData::new(api_key).to_header_map();

    let client: Client = Client::new();
    let response: Response = client.get(&url).headers(headers).send().await?;

    if response.status().is_success() {
      let price_response: PriceResponse = response.json().await?;
      let prices : Vec<Price> = price_response.prices;


      if !prices.is_empty() {
        // Convert Vec<Price> to Vec<HashMap<String, Value>> for the current cache structure
        let data_to_cache_maps: Vec<HashMap<String, Value>> = prices.iter().filter_map(|p_struct| {
          match serde_json::to_value(p_struct) { // Price to serde_json::Value
            Ok(json_val) => match serde_json::from_value(json_val) { // Value to HashMap
              Ok(h_map) => Some(h_map),
              Err(e) => {
                log::error!("Failed to deserialize Price to HashMap for caching {}: {}", ticker, e);
                None
              }
            },
            Err(e) => {
              log::error!("Failed to serialize Price to Value for caching {}: {}", ticker, e);
              None
            }
          }
        }).collect();
      
      
        if !data_to_cache_maps.is_empty() {
          let mut cache_guard = cache.lock().unwrap(); // Re-acquire lock for writing
          if let Err(e) = cache_guard.set_prices(ticker, data_to_cache_maps) {
            log::error!("Error saving prices to cache for ticker {}: {}",ticker,e);
          } else {
            log::info!("Prices for ticker {} saved to cache.", ticker);
          }
        } 
      }

      return Ok(prices);
    } 
    else {
      log::error!("Error getting prices for a specific company: {} with status code: {}", ticker, response.status());
      return Err(response.error_for_status().unwrap_err());
    }
  }


  pub async fn get_financial_metrics(&self, ticker: &str, end_date: &str, period: Option<&str>, limit: Option<i64>) -> Result<Vec<FinancialMetrics>, Error> {
    let period: &str = period.unwrap_or("ttm");
    let limit : i64 = limit.unwrap_or(10);

    let cache : &'static Mutex<Cache> = cache::get_cache();

    {
      let cache_guard  = cache.lock().unwrap(); 
      let result = cache_guard.get_financial_metrics(ticker);

      match result {
        Ok(data) if !data.is_empty() => {
          let metrics : Vec<FinancialMetrics> = data.into_iter().filter_map( |h_map|{
            match serde_json::to_value(h_map) {
              Ok(json_value) => match serde_json::from_value(json_value) {
                Ok(price_struct) => Some(price_struct), 
                Err(e) => {
                  log::warn!("Failed to deserialize cached price item for {}: {}", ticker, e);
                  None
                }
              }, 
              Err(e) => {
                log::warn!("Failed to convert cached HashMap to Value for {}: {}", ticker, e);
                None
              }
            }
          }).collect();

          if !metrics.is_empty() {
            log::info!("Returning prices for ticker {} from cache.", ticker);
            // TODO: Optionally filter 'prices' by start_date and end_date if cache stores more than requested.
            return Ok(metrics);
          }
          else {
            log::info!("Cached data for {} was empty or failed deserialization.", ticker);
          }
        }, 
        Ok(_) => {
          log::info!("Cache miss (empty data) for prices (ticker: {}).", ticker);
        }, 
        Err(e) => {
          log::error!("Error accessing cache for prices (ticker: {}): {}. Proceeding to API call.",ticker,e);
        }
      }
    }

    let url : String = format!("https://api.financialdatasets.ai/financial-metrics/?ticker={}&report_period_lte={}&limit={}&period={}", ticker, end_date, limit, period);
    let api_key: String = self.config.financial_datasets_api_key.clone();
    let headers: HeaderMap = FinancialHeaderData::new(api_key).to_header_map();

    let client : Client = Client::new();

    let response : Response = client.get(&url).headers(headers).send().await?;

    if response.status().is_success() {
      let metric_response : FinancialMetricsResponse = response.json().await?;
      let metrics: Vec<FinancialMetrics> = metric_response.financial_metrics;

      if !metrics.is_empty() {
        // Convert Vec<Price> to Vec<HashMap<String, Value>> for the current cache structure
        let data_to_cache_maps: Vec<HashMap<String, Value>> = metrics.iter().filter_map(|p_struct| {
          match serde_json::to_value(p_struct) { // Price to serde_json::Value
            Ok(json_val) => match serde_json::from_value(json_val) { // Value to HashMap
              Ok(h_map) => Some(h_map),
              Err(e) => {
                log::error!("Failed to deserialize Price to HashMap for caching {}: {}", ticker, e);
                None
              }
            },
            Err(e) => {
              log::error!("Failed to serialize Price to Value for caching {}: {}", ticker, e);
              None
            }
          }
        }).collect();
      
      
        if !data_to_cache_maps.is_empty() {
          let mut cache_guard = cache.lock().unwrap(); // Re-acquire lock for writing
          if let Err(e) = cache_guard.set_financial_metrics(ticker, data_to_cache_maps) {
            log::error!("Error saving prices to cache for ticker {}: {}",ticker,e);
          } else {
            log::info!("Prices for ticker {} saved to cache.", ticker);
          }
        } 
      }

      return Ok(metrics);
    }
    else {
      log::error!("Error getting prices for a specific company: {}", ticker);
      return Err(response.error_for_status().unwrap_err());
    }

  }


  pub async fn search_line_items(&self, ticker: &str, line_items: Vec<String>, end_date : &str, period: Option<&str>, limit: Option<i64>) -> Result<Vec<LineItem>, Error> {
    let period: &str = period.unwrap_or("ttm");
    let limit : i64 = limit.unwrap_or(10);

    let limit_usize : usize = limit as usize;

    let url : &'static str = "https://api.financialdatasets.ai/financials/search/line-items";

    let api_key: String = self.config.financial_datasets_api_key.clone();
    let headers: HeaderMap = FinancialHeaderData::new(api_key).to_header_map();

    let body : LineItemBodyData = LineItemBodyData { tickers: vec![ticker.to_string()], line_items:line_items, end_date: end_date.to_string(), period: period.to_string(), limit: limit };

    let client : Client = Client::new(); 

    let response : Response = client.post(url).headers(headers).json(&body).send().await?;

    if response.status().is_success() {
      let line_response : LineItemResponse = response.json().await?; 
      if line_response.search_results.is_empty() {
        return Ok(Vec::new()); 
      }

      let limited_results: Vec<LineItem> = line_response.search_results.into_iter().take(limit_usize).collect();
      return Ok(limited_results);
    }
    else {
      log::error!("Error searching line items for ticker {}: API request failed with status {}",ticker,response.status());
      Err(response.error_for_status().unwrap_err())
    }
    
  }

  pub async fn get_insider_trade(&self, ticker: &str, end_date : &str, start_date: Option<&str>, limit: i64 ) -> Result<Vec<InsiderTrade>, Error> {
    let target_end_date = match NaiveDate::parse_from_str(end_date, "%Y-%m-%d") {
      Ok(d) => d,
      Err(e) => {
        log::error!("Invalid end_date format for market cap: {}", e);
        return Ok(Vec::new());  // or `return Err(/* some reqwest::Error */)` if you prefer
      }
    };

    let target_start_date_opt: Option<NaiveDate> = start_date
      .map(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d"))
      .transpose() // Result<Option<NaiveDate>, ParseError>
      .unwrap_or_else(|e| {
        log::error!("Invalid start_date format for news: {}", e);
        None  // on parse error, just treat as “no start date” 
    });

    let cache_mutex = cache::get_cache();

    {
      let cache_guard = cache_mutex.lock().unwrap_or_else(|p| p.into_inner());
      if let Ok(cached_maps) = cache_guard.get_insider_trades(ticker) {
        if !cached_maps.is_empty() {
          let mut trades: Vec<InsiderTrade> = cached_maps
            .into_iter()
            .filter_map(|h_map| self.convert_cached_item_to_model(h_map, "InsiderTrade", ticker)).collect();

          // Filter by date range
          trades.retain(|trade| {
            let trade_date_str = trade.transaction_date.as_deref().or(trade.filing_date.as_deref()) .unwrap_or_default(); 
            if let Ok(trade_date) = NaiveDate::parse_from_str(trade_date_str.split('T').next().unwrap_or(""), "%Y-%m-%d") {
                let after_start = target_start_date_opt.map_or(true, |start| trade_date >= start);
                let before_end = trade_date <= target_end_date;
                return after_start && before_end;
            }
            false
          });

          // Sort
          trades.sort_by(|a, b| {
            let date_a = a.transaction_date.as_deref().or(a.filing_date.as_deref()).unwrap_or_default();
            let date_b = b.transaction_date.as_deref().or(b.filing_date.as_deref()).unwrap_or_default();
            date_b.cmp(date_a) // reverse=True
          });

          if !trades.is_empty() {
            log::info!("Returning insider trades for {} from cache after filtering.", ticker);
            return Ok(trades);
          }
        }
      }
    }


    log::info!("Fetching insider trades for {} from API.", ticker);
    let mut all_fetched_trades: Vec<InsiderTrade> = Vec::new();
    let mut current_page_end_date_str: String = end_date.to_string();
    let client: Client = Client::new();

    loop {
      let mut url = format!(
        "https://api.financialdatasets.ai/insider-trades/?ticker={}&filing_date_lte={}&limit={}",
        ticker, current_page_end_date_str, limit
      );
      if let Some(start_date_val_str) = start_date {
        url.push_str(&format!("&filing_date_gte={}", start_date_val_str));
      }

      let mut headers = HeaderMap::new();
      if let Ok(api_key) = env::var("FINANCIAL_DATASETS_API_KEY") {
        if let Ok(header_val) = HeaderValue::from_str(&api_key) {
          headers.insert("X-API-KEY", header_val);
        }
      }

      log::debug!("Fetching insider trades from URL: {}", url);
      let response = client.get(&url).headers(headers).send().await?;

      let mut current_batch_trades: Vec<InsiderTrade> = Vec::new(); 

      if response.status().is_success() {
        let response_model: InsiderTradeResponse = response.json().await?;
        current_batch_trades = response_model.insider_trades;
      }

      else if !response.status().is_success() {
        let err_text = response.text().await.unwrap_or_default();
        log::error!("API error for insider trades ({}): - {}", ticker, err_text);
      }

      if current_batch_trades.is_empty() {
        break; // No more data
      }

      all_fetched_trades.extend(current_batch_trades.clone()); // Clone needed if we use current_batch_trades later

      // Only continue pagination if we have a start_date and got a full page
      if start_date.is_none() || current_batch_trades.len() < limit as usize {
        break;
      }

      // Update end_date to the oldest filing date from current batch for next iteration
      if let Some(oldest_trade_in_batch) = current_batch_trades.iter().min_by_key(|t| t.filing_date.as_deref().unwrap_or_default()) {
        if let Some(filing_date_str) = &oldest_trade_in_batch.filing_date {
          current_page_end_date_str = filing_date_str.split('T').next().unwrap_or("").to_string();
          if let (Some(start_date_val), Ok(current_page_end_naive_date)) = (target_start_date_opt, NaiveDate::parse_from_str(&current_page_end_date_str, "%Y-%m-%d")) {
            if current_page_end_naive_date <= start_date_val {
              break; // Reached or passed the overall start_date
            }
          } else if target_start_date_opt.is_none() {
            // If no start date, and we got here, it means we are paginating.
            // If current_page_end_date_str becomes empty or invalid, break.
            if current_page_end_date_str.is_empty() { break; }
          }
        } else {
          break; // Cannot determine next page end date
        }
      } else {
        break; // Should not happen if current_batch_trades was not empty
      }
    }

    if all_fetched_trades.is_empty() {
      return Ok(Vec::new());
    } else {
      return Ok(all_fetched_trades);
    }
  } 

  pub async fn get_company_news(&self,ticker: &str,end_date_str: &str,start_date_opt: Option<&str>,limit_per_page: i64,) -> Result<Vec<CompanyNews>, Error> {
    let target_end_date = match NaiveDate::parse_from_str(end_date_str, "%Y-%m-%d") {
      Ok(d) => d,
      Err(e) => {
        log::error!("Invalid end_date format for market cap: {}", e);
        return Ok(Vec::new());  // or `return Err(/* some reqwest::Error */)` if you prefer
      }
    };

    let target_start_date_opt: Option<NaiveDate> = start_date_opt
      .map(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d"))
      .transpose() // Result<Option<NaiveDate>, ParseError>
      .unwrap_or_else(|e| {
        log::error!("Invalid start_date format for news: {}", e);
        None  // on parse error, just treat as “no start date” 
    });

    let cache_mutex = cache::get_cache();

    // 1. Check cache
    {
      let cache_guard = cache_mutex.lock().unwrap_or_else(|p| p.into_inner());
      if let Ok(cached_maps) = cache_guard.get_company_news(ticker) {
          if !cached_maps.is_empty() {
            let mut news_items: Vec<CompanyNews> = cached_maps.into_iter().filter_map(|h_map| self.convert_cached_item_to_model(h_map, "CompanyNews", ticker)).collect();

            // Filter by date range
            news_items.retain(|news| {
              if let Ok(news_date) = NaiveDate::parse_from_str(news.date.split('T').next().unwrap_or(""), "%Y-%m-%d") {
                let after_start = target_start_date_opt.map_or(true, |start| news_date >= start);
                let before_end = news_date <= target_end_date;
                return after_start && before_end;
              }
              false
            });

            // Sort
            news_items.sort_by(|a, b| b.date.cmp(&a.date)); // reverse=True

            if !news_items.is_empty() {
                log::info!("Returning company news for {} from cache after filtering.", ticker);
                return Ok(news_items);
            }
        }
      }
    }

    // 2. Fetch from API with pagination
    log::info!("Fetching company news for {} from API.", ticker);
    let mut all_fetched_news: Vec<CompanyNews> = Vec::new();
    let mut current_page_end_date_str: String = end_date_str.to_string(); // API uses 'end_date' for news
    let client = Client::new();

    loop {
      let mut url = format!(
        "https://api.financialdatasets.ai/news/?ticker={}&end_date={}&limit={}", // API endpoint for news
        ticker, current_page_end_date_str, limit_per_page
      );
      if let Some(start_date_val_str) = start_date_opt {
        url.push_str(&format!("&start_date={}", start_date_val_str)); // API uses 'start_date'
      }

      let mut headers = HeaderMap::new();
        if let Ok(api_key) = env::var("FINANCIAL_DATASETS_API_KEY") {
          if let Ok(header_val) = HeaderValue::from_str(&api_key) {
            headers.insert("X-API-KEY", header_val);
          }
      }

      log::debug!("Fetching company news from URL: {}", url);
      let response = client.get(&url).headers(headers).send().await?;

      let mut current_batch_news : Vec<CompanyNews> = Vec::new();

      if response.status().is_success() { 
        let response_model: CompanyNewsResponse = response.json().await?;
        current_batch_news = response_model.news;
      }

      else {
        let err_text = response.text().await.unwrap_or_default();
        log::error!("API error for company news ({}):  - {}", ticker, err_text);
      }

      

      all_fetched_news.extend(current_batch_news.clone());

      if start_date_opt.is_none() || current_batch_news.len() < limit_per_page as usize {
          break;
      }
      
      if let Some(oldest_news_in_batch) = current_batch_news.iter().min_by_key(|n| &n.date) {
        current_page_end_date_str = oldest_news_in_batch.date.split('T').next().unwrap_or("").to_string();
        if let (Some(start_date_val), Ok(current_page_end_naive_date)) = (target_start_date_opt, NaiveDate::parse_from_str(&current_page_end_date_str, "%Y-%m-%d")) {
            if current_page_end_naive_date <= start_date_val {
                break;
            }
        } else if target_start_date_opt.is_none() {
            if current_page_end_date_str.is_empty() { break; }
        }
      } else {
        break;
      }
    }

    if all_fetched_news.is_empty() {
      return Ok(Vec::new());
    }

    // 3. Cache results
    let data_to_cache: Vec<HashMap<String, Value>> = all_fetched_news.iter().filter_map(|news| self.convert_model_to_cache_item(news, "CompanyNews", ticker)).collect();

    if !data_to_cache.is_empty() {
      let mut cache_guard = cache_mutex.lock().unwrap_or_else(|p| p.into_inner());
      if let Err(e) = cache_guard.set_company_news(ticker, data_to_cache) {
        log::error!("Failed to cache company news for {}: {}", ticker, e);
      } else {
        log::info!("Cached company news for {}.", ticker);
      }
    }
    Ok(all_fetched_news)
  }

  pub async fn get_market_cap(&self,ticker: &str,end_date: &str,) -> Result<Option<f64>, Error> { // "YYYY-MM-DD" // Market cap can be None
    let target_end_date = match NaiveDate::parse_from_str(end_date, "%Y-%m-%d") {
      Ok(d) => d,
      Err(e) => {
          log::error!("Invalid end_date format for market cap: {}", e);
          return Ok(None);  // or `return Err(/* some reqwest::Error */)` if you prefer
      }
    };
    
    let today = chrono::Local::now().date_naive();

    if target_end_date == today {
      log::info!("Fetching market cap for {} from company facts (today's date).", ticker);
      let url = format!("https://api.financialdatasets.ai/company/facts/?ticker={}", ticker);
      
      let mut headers = HeaderMap::new();
      if let Ok(api_key) = env::var("FINANCIAL_DATASETS_API_KEY") {
        if let Ok(header_val) = HeaderValue::from_str(&api_key) {
          headers.insert("X-API-KEY", header_val);
        }
      }

      let client: Client = Client::new();
      let response: Response = client.get(&url).headers(headers).send().await?;

      if response.status().is_success() {
        // Assuming CompanyFactsResponse and CompanyFacts models are defined
        let facts_response: CompanyFactsResponse = response.json().await?;
        return Ok(facts_response.company_facts.market_cap);
      } 
      else {
        log::error!("Error fetching company facts for market cap ({}): {}",ticker, response.status());
        // Fall through to try financial_metrics if appropriate, or return error/None
        // For now, let's return None on API error here to match Python's print then None
        return Ok(None);
      }
    }
    return Ok(None);
  }


  pub fn prices_to_df(&self, prices: Vec<Price>) -> anyhow::Result<DataFrame> {
    if prices.is_empty() {
      let df = DataFrame::new(vec![
        Series::new("open",   &Vec::<f64>::new()),
        Series::new("close",  &Vec::<f64>::new()),
        Series::new("high",   &Vec::<f64>::new()),
        Series::new("low",    &Vec::<f64>::new()),
        Series::new("volume", &Vec::<i64>::new()),
        Series::new("time",   &Vec::<String>::new()),
      ])?;

      return Ok(df);
    }

    let opens:   Vec<f64>   = prices.iter().map(|p| p.open).collect();
    let closes:  Vec<f64>   = prices.iter().map(|p| p.close).collect();
    let highs:   Vec<f64>   = prices.iter().map(|p| p.high).collect();
    let lows:    Vec<f64>   = prices.iter().map(|p| p.low).collect();
    let volumes: Vec<i64>   = prices.iter().map(|p| p.volume).collect();
    let times:   Vec<String>= prices.iter().map(|p| p.time.clone()).collect();

    let mut df = DataFrame::new(vec![
      Series::new("open",   &opens),
      Series::new("close",  &closes),
      Series::new("high",   &highs),
      Series::new("low",    &lows),
      Series::new("volume", &volumes),
      Series::new("time",   &times),
    ])?;

    let ca = df.column("time")?.str()?;
    let tz_string = "Australia/Perth".to_string();
    let tz = Some(&tz_string);

    let date_series = ca.as_datetime(
      Some("%Y-%m-%dT%H:%M:%S"),   // fmt
      TimeUnit::Milliseconds,  
      false, 
      false, 
      tz, 
      ca)?.clone();               // then cast to Date

    let mut series = date_series.into_series(); 
    
    series.rename("Date");    // now a Series       // consumes & returns Series

    // and finally add it
    df.with_column(series)?;
    // slice of column names defaults to ascending, nulls_first, no stability  single‐column API wants &str
    let df = df.sort(&["Date"],SortMultipleOptions::default())?; 
    Ok(df)
  }


  pub async fn get_price_data(&self, ticker: &str, start_date: &str, end_date: &str ) -> anyhow::Result<DataFrame> {
    let prices: Vec<Price> =  self.get_price(ticker, start_date, &end_date).await?;

    let df: DataFrame = self.prices_to_df(prices)?;

    return Ok(df);
  }

  pub fn convert_model_to_cache_item(&self, news: &CompanyNews, _type_tag: &str, _ticker: &str ) -> Option<HashMap<String, Value>> {    // unused, but keeps the interface consistent
    // 1) Serialize the model to a serde_json::Value
    let val = serde_json::to_value(news).ok()?;
    // 2) Expect it to be an Object and clone into a HashMap
    val.as_object()
      .cloned()
      .map(|map| map.into_iter().collect())
  }

  pub fn convert_cached_item_to_model<T: DeserializeOwned>(&self, map: HashMap<String, Value>, _type_tag: &str, _ticker: &str) -> Option<T> {
    // unused, but keeps your signature consistent
    // Rebuild a serde_json::Value::Object, then
    // attempt to deserialize into your model
    let json_map: serde_json::Map<String, Value> = map.into_iter().collect();
    // Wrap it back into a Value::Object
    let value = Value::Object(json_map);
    // Then try to deserialize
    serde_json::from_value(value).ok()
  }

}