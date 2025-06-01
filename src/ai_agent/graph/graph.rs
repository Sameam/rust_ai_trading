// src/ai_agent/graph/graph.rs
use std::collections::{HashMap, HashSet};
use async_trait::async_trait;
use anyhow::{Result, Error};
use std::sync::Arc;
use std::future::Future; 
use std::pin::Pin;

use crate::ai_agent::graph::state::{AgentState, PartialAgentStateUpdate};
use crate::app::config::Config;

// Define a trait for node functions
#[async_trait]
pub trait NodeFunction: Send + Sync {
  async fn call(&self, state: AgentState, config: Config) -> Result<PartialAgentStateUpdate>;
}

// Allow Fn types to be used as NodeFunction
#[async_trait]
impl<F> NodeFunction for F where F: Fn(AgentState, Config) -> Pin<Box<dyn Future<Output = Result<PartialAgentStateUpdate, Error>> + Send>> + Send + Sync,
{
    async fn call(&self, state: AgentState, config: Config) -> Result<PartialAgentStateUpdate> {
      let future = self(state, config);
      future.await
    }
}

pub struct StateGraph {
  nodes: HashMap<String, Box<dyn NodeFunction>>,
  edges: HashMap<String, Vec<String>>,
  entry_point: Option<String>,
  end_node: String,
}

impl StateGraph {
  pub fn new() -> Self {
    StateGraph {
      nodes: HashMap::new(),
      edges: HashMap::new(),
      entry_point: None,
      end_node: "END".to_string(),
    }
  }

  pub fn add_node<F>(&mut self, name: String, func: F) where F: NodeFunction + 'static, {
    self.nodes.insert(name.clone(), Box::new(func));
    // Initialize empty edge list for this node
    if !self.edges.contains_key(&name) {
      self.edges.insert(name.to_string(), Vec::new());
    }
  }

  pub fn add_edge(&mut self, from: String, to: String) {
    self.edges.entry(from).or_insert_with(Vec::new).push(to);
  }

  pub fn set_entry_point(&mut self, node: &str) {
    self.entry_point = Some(node.to_string());
  }

  pub fn compile(self) -> CompiledGraph {
    CompiledGraph { graph: Arc::new(self) }
  }
}

#[derive(Clone)]
pub struct CompiledGraph {
  graph: Arc<StateGraph>,
}

impl CompiledGraph {
  pub async fn invoke(&self, initial_state: AgentState, config: Config) -> Result<AgentState> {
    let mut current_state = initial_state;
    let mut current_node = self.graph.entry_point.clone().expect("Graph must have an entry point");
    
    let mut visited = HashSet::new();
    
    while current_node != self.graph.end_node {
      // Prevent infinite loops
      if visited.contains(&current_node) {
        return Err(anyhow::anyhow!("Cycle detected in graph execution"));
      }
      visited.insert(current_node.clone());
      
      // Get the node function
      let node_func = self.graph.nodes.get(&current_node).ok_or_else(|| anyhow::anyhow!("Node not found: {}", current_node))?;
      
      // Call the node function
      let update = node_func.call(current_state.clone(), config.clone()).await?;
      
      // Update the state
      current_state.update_from_partial(update)?;
      
      // Get next node
      let next_nodes = self.graph.edges.get(&current_node).ok_or_else(|| anyhow::anyhow!("No edges defined for node: {}", current_node))?;
      
      if next_nodes.is_empty() {
        return Err(anyhow::anyhow!("Dead end at node: {}", current_node));
      }
      
      // For simplicity, just take the first edge
      // In a more complex system, you might have conditional routing
      current_node = next_nodes[0].clone();
    }
    
    Ok(current_state)
  }
}
