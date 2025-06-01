use std::collections::HashMap;
use serde_json::Value;
use anyhow::{Result, anyhow, Context, Error};
use std::result::Result::{Ok, Err};
use std::future::Future; 
use std::pin::Pin;

use crate::ai_agent::agents::portfolio_manager::PortfolioManagerAgent;
use crate::ai_agent::agents::risk_manager::RiskManagerAgent;
use crate::ai_agent::llm::model_provider::ChatMessage;
use crate::app::config::Config;
use crate::ai_agent::graph::graph::{CompiledGraph, StateGraph};
use crate::ai_agent::graph::state::{AgentState, PartialAgentStateUpdate};
use crate::ai_agent::utils::analysts::{get_analyst_config, get_analyst_nodes};

pub struct AgentService {
  config : Config,
  default_agent : Option<CompiledGraph>
}

impl AgentService {
  pub fn new(config: Config) -> Self {
    let temp_agent: AgentService = AgentService {
      config: config.clone(), 
      default_agent: None
    };
    let default_workflow: StateGraph = temp_agent.create_workflow(None);  // Create workflow with all analysts
    let default_agent = Some(default_workflow.compile());
    AgentService { config, default_agent }
  }

  pub async fn run_hedge_fund(&self, ticker: Vec<String>, start_date: &str, end_date: &str, portfolio: HashMap<String, Value>, 
                              show_reasoning: Option<bool>, selected_analysts: Option<Vec<String>>, 
                              model_name: Option<&str>, model_provider: Option<&str>) -> std::result::Result<HashMap<String, Value>, Error> {
    
    let show_reasoning : bool = show_reasoning.unwrap_or(false);
    let selected_analysts : Vec<String> = selected_analysts.unwrap_or(Vec::new());
    let model_name : &str = model_name.unwrap_or("gpt-4o");
    let model_provider : &str = model_provider.unwrap_or("OpenAI");

    let result = {
      let agent: CompiledGraph  = if !selected_analysts.is_empty() {
        let workflow : StateGraph = self.create_workflow(Some(selected_analysts.clone())); 
        let agent : CompiledGraph = workflow.compile();
        agent
      }
      else if let Some(default) = self.default_agent.as_ref() {
        let agent = default.clone();
        agent
      }
      else {
        return Err(anyhow!("No default agent available"));
      };

      let mut initial_state: AgentState = AgentState::new(); 
      let _ = initial_state.add_message(ChatMessage {
        role: "user".to_string(), content: "Make trading decisions based on the provided data.".to_string(),
      });

      let mut data: HashMap<String, Value> = HashMap::new(); 
      data.insert("tickers".to_string(), serde_json::to_value(&ticker)?);
      data.insert("portfolio".to_string(), serde_json::to_value(&portfolio)?); 
      data.insert("start_date".to_string(), serde_json::to_value(start_date)?);
      data.insert("end_date".to_string(), serde_json::to_value(end_date)?);
      data.insert("analyst_signals".to_string(), serde_json::json!({}));
      let _ = initial_state.merge_data(data);

      let mut meta_data: HashMap<String, Value> = HashMap::new(); 
      meta_data.insert("show_reasoning".to_string(), serde_json::to_value(show_reasoning)?);
      meta_data.insert("model_name".to_string(), serde_json::to_value(model_name)?);
      meta_data.insert("model_provider".to_string(), serde_json::to_value(model_provider)?);
      let _ = initial_state.merge_metadata(meta_data);

      let final_state : AgentState = agent.invoke(initial_state, self.config.clone()).await?;

      let last_message = final_state.messages.last().context("No messages in final state")?;
            
      let decisions = self.parse_hedge_fund_response(&last_message.content)?;
      let analyst_signals = final_state.data.get("analyst_signals").cloned().unwrap_or_else(|| serde_json::json!({}));
      
      // Return the results
      let mut result = HashMap::new();
      result.insert("decisions".to_string(), decisions);
      result.insert("analyst_signals".to_string(), analyst_signals);
      
      Ok(result)

    };

    return result;


  }

  pub fn start(_state: AgentState, _config: Config) -> Pin<Box<dyn Future<Output = Result<PartialAgentStateUpdate, Error>> + Send>> {
    Box::pin(async move {
        Ok(PartialAgentStateUpdate::new())
    })
  }


  fn create_workflow(&self, selected_analyst: Option<Vec<String>>) -> StateGraph {
    let mut workflow: StateGraph = StateGraph::new(); 

    workflow.add_node("start_node".to_string(), Self::start);

    let analyst_nodes = get_analyst_nodes();
    

    let selected_analysts = match &selected_analyst {
      Some(selected) if !selected.is_empty() => selected.clone(), 
      _ => analyst_nodes.keys().cloned().collect(),
    };

    for analyst_key in &selected_analysts {
      if let Some((node_name, node_function)) = analyst_nodes.get(analyst_key) {
        workflow.add_node(node_name.to_string(), *node_function);
        workflow.add_edge("start_node".to_string(), node_name.to_string());
      }
    }

    workflow.add_node("risk_management_agent".to_string(), RiskManagerAgent::static_risk_management_agent);
    workflow.add_node("portfolio_manager".to_string(), PortfolioManagerAgent::static_portfolio_management_agent);

    for analyst_key in &selected_analysts {
      if let Some((node_name, _node_function)) = analyst_nodes.get(analyst_key) {
         workflow.add_edge(node_name.to_string(), "risk_management_agent".to_string());
      }
    }

    workflow.add_edge("risk_management_agent".to_string(), "portfolio_manager".to_string());
    workflow.add_edge("portfolio_manager".to_string(), "END".to_string());
    workflow.set_entry_point("start_node");


    return workflow;
  }


  fn parse_hedge_fund_response(&self, response: &str) -> Result<Value> {
    match serde_json::from_str(response) {
      Ok(value) => Ok(value),
      Err(e) => {
        log::error!("JSON decoding error: {}. Response: {:?}", e, response);
        Err(anyhow!("Failed to parse hedge fund response: {}", e))
      }
    }
  }

}

