use std::collections::HashMap;
use serde_json::Value; 
use anyhow::{Result, Error};
use std::future::Future;
use std::pin::Pin;

use crate::ai_agent::agents::warren_buffet::{Signal, WarrenBuffetSignal};
use crate::ai_agent::graph::state::{PartialAgentStateUpdate, AgentState};
use crate::app::config::Config; 

pub type AgentFunction = fn(AgentState, Config) -> Pin<Box<dyn Future<Output = Result<PartialAgentStateUpdate, Error>> + Send>>;
pub type NodeFunctionPair = (String, AgentFunction);

pub struct AnalystConfig {
  pub display_name: String,
  pub agent_function : AgentFunction,
  pub order : usize,
}

pub fn get_analyst_config() -> HashMap<String, AnalystConfig> {
  let mut config: HashMap<String, AnalystConfig> = HashMap::new();

  config.insert("warren_buffett".to_string(), AnalystConfig { 
    display_name: "Warren Buffett".to_string(), 
    agent_function: WarrenBuffetSignal::static_warren_buffet_agent, 
    order: 8 
  });

  return config;
}


pub fn get_analyst_order() -> Vec<(String, String)> {
  let config = get_analyst_config();
  let mut order_vec: Vec<(String, String)> = Vec::new();
  
  // Create a vector of (key, config) pairs
  let mut config_pairs: Vec<(String, &AnalystConfig)> = config.iter().map(|(k, v)| (k.clone(), v)).collect();
  
  // Sort by order
  config_pairs.sort_by_key(|(_, config)| config.order);
  
  // Transform into (display_name, key) pairs
  for (key, config) in config_pairs {
    order_vec.push((config.display_name.clone(), key));
  }
  
  return order_vec;
}

pub fn get_analyst_nodes() -> HashMap<String, NodeFunctionPair> {
  let config = get_analyst_config();
  let mut nodes = HashMap::new();
  
  for (key, config) in config.iter() {
    nodes.insert(key.clone(),(format!("{}_agent", key), config.agent_function));
  }
  
  return nodes;
}
