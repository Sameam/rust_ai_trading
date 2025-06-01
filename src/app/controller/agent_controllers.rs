use std::result::Result::{Ok};
use std::sync::Arc;
use std::collections::HashMap; 
use anyhow::Error;
use serde_json::Value; 

use crate::app::services;
use crate::app::services::service::{HedgeFundServices};

pub struct AgentController {
  services : Arc<HedgeFundServices>
}

impl AgentController {
  pub fn new(services: Arc<HedgeFundServices>) -> Self {
    AgentController {services: services}
  }

  pub async fn get_available_analysts(&self) -> Result<Vec<HashMap<String, String>>, Error> {
    let analysts = match self.services.get_available_analysts() {
      Ok(analysts) => analysts,
      Err(e) => {
        let error: Vec<HashMap<String, String>> = Vec::new();
        log::error!("Cannot find an analysts with error: {}", e);
        error
      },
    };
    return Ok(analysts);
  }

  pub async fn get_available_model(&self) -> Result<(Vec<HashMap<String, String>>, Vec<HashMap<String, String>>), Error> {
    let models = match self.services.get_available_models() {
      Ok(models) => models, 
      Err(e) => {
        let error1: Vec<HashMap<String, String>> = Vec::new();
        let error2: Vec<HashMap<String, String>> = Vec::new();
        log::error!("Cannot find an analysts with error: {}", e);
        (error1, error2)
      }
    }; 

    return Ok(models);
  }

  pub async fn hedge_fund(&self, tickers: Vec<String>, start_date: Option<&str>, end_date: Option<&str>, 
                          initial_cash: Option<f64>, margin_requirement: Option<f64>, show_reasoning: Option<bool>, 
                          selected_analysts: Option<Vec<String>>, model_name: Option<String>, model_provider: Option<String>) -> Result<HashMap<String, Value>, Error> {

    let result =  match self.services.hedge_fund(tickers, start_date, end_date, initial_cash, margin_requirement, show_reasoning, selected_analysts, model_name, model_provider).await {
      Ok(data) => data,
      Err(e) => {
        log::error!("Cannot find an analysts with error: {}", e);
        let error: HashMap<String, Value> = HashMap::new();
        error
      }
    };

    return Ok(result);
  }

}