use std::collections::HashMap; 
use serde::{Serialize, Deserialize}; 
use serde_json::Value; 
use log; 
use std::result::Result::{Ok, Err};
use anyhow::Error;

use crate::ai_agent::llm::model_provider::ChatMessage; 

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AgentState {
  pub messages : Vec<ChatMessage>, 
  pub data : HashMap<String, Value>, 
  pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)] // Added Default
pub struct PartialAgentStateUpdate {
  pub messages: Option<Vec<ChatMessage>>,
  pub data: Option<HashMap<String, Value>>,
  pub metadata: Option<HashMap<String, Value>>,
}


impl AgentState {
  pub fn new() -> Self {
    AgentState {
      messages: Vec::new(), data: HashMap::new(), metadata: HashMap::new(),
    }
  }

  pub fn add_messages(&mut self, messages: Vec<ChatMessage>) -> Result<(), Error>{
    let _ = self.messages.extend(messages);
    // Optionally log success
    log::info!("Extended messages vector correctly.");
    // Explicitly return Ok(()) to satisfy the Result<(), Error> signature
    return Ok(());
  }

  pub fn add_message(&mut self, messages: ChatMessage) -> Result<(), Error> {
    let _ = self.messages.push(messages);
    log::info!("Push one message into a vector correctly");
    return Ok(());
  }

  pub fn merge_data(&mut self, data: HashMap<String, Value>) -> Result<(), Error> {
    let _ = self.data.extend(data);
    log::info!("Merge data into a dictionary correctly");
    return Ok(());
  }

  pub fn merge_metadata(&mut self, data: HashMap<String, Value>) -> Result<(), Error> {
    let _ = self.metadata.extend(data);
    log::info!("Merge meta_data into a dictionary correctly");
    return Ok(());
  }

  pub fn update_from_partial(&mut self, update: PartialAgentStateUpdate) -> Result<(), Error> {
    if let Some(new_messages) = update.messages {
      let _ = self.add_messages(new_messages);
    }

    if let Some(new_data) = update.data {
      let _ = self.merge_data(new_data);
    }

    if let Some(new_metadata) = update.metadata {
      let _ = self.merge_metadata(new_metadata);
    }

    return Ok(());
  }


}


impl PartialAgentStateUpdate {
  pub fn new() -> Self {
    PartialAgentStateUpdate { messages: None, data: None, metadata: None }
  }

  pub fn with_messages(mut self, messages: Vec<ChatMessage>) -> Self {
    self.messages = Some(messages); 
    return self; 
  }

  pub fn with_data(mut self, data: HashMap<String, Value>) -> Self {
    self.data = Some(data); 
    return self;
  }

  pub fn with_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
    self.metadata = Some(metadata);
    return self;
  }
}

pub fn show_agent_reasoning(output_str: &str, agent_name: &str) {
  log::info!("\n{:=<10} {:^28} {:=<10}", "", agent_name, "");

  match serde_json::from_str::<serde_json::Value>(output_str) {
    Ok(json_value) => { // Successfully parsed the string as JSON
      match serde_json::to_string_pretty(&json_value) {
        Ok(pretty_json_string) => log::info!("{}", pretty_json_string),
        Err(e) => {
          log::error!("Failed to re-serialize parsed JSON for '{}': {}. Printing raw parsed value.", agent_name, e);
          log::info!("{:?}", json_value); // Print the parsed Value if re-serialization fails
        }
      }
    }
    Err(_) => { // Not a valid JSON string, print as is
      log::info!("{}", output_str);
    }
  }
  log::info!("{:=<48}", "");
}