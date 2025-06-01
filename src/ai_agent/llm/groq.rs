use crate::ai_agent::llm::model_provider::{ChatMessage, LLMChatter, LLMModelConfig, LLMResponse}; 

use reqwest::{header::{HeaderMap},Client, Response};
use serde::{Deserialize, Serialize};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use std::result::Result::Ok;


#[derive(Serialize, Debug)]
struct GroqChatRequest {
  messages: Vec<ChatMessage>, // Reusing the generic ChatMessage from model_provider
  model: String,              // e.g., "meta-llama/llama-4-scout-17b-16e-instruct"
  #[serde(skip_serializing_if = "Option::is_none")]
  temperature: Option<f32>,
  #[serde(rename = "max_tokens")] // OpenAI compatible APIs often use max_tokens
  #[serde(skip_serializing_if = "Option::is_none")]
  max_completion_tokens: Option<u32>, // Matching curl's "max_completion_tokens"
  #[serde(skip_serializing_if = "Option::is_none")]
  top_p: Option<f32>,
  // stream: bool, // For this example, we'll assume non-streaming. Set to false or omit.
  // stop: Option<Vec<String>>, // Example: stop: Some(vec!["\n".to_string()])
}

#[derive(Deserialize, Debug)]
struct GroqResponseMessage {
  // role: String, // Usually "assistant"
  content: String,
}

#[derive(Deserialize, Debug)]
struct GroqChoice {
  // index: u32,
  message: GroqResponseMessage,
  // finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
struct GroqChatResponse {
  choices: Vec<GroqChoice>,
  // You could add other fields like 'id', 'usage', etc., if needed.
}

pub struct GroqProvider {
  groq_url : String,
  api_key : String,
  model_name: String,
  client : Client
}

impl GroqProvider {

  pub fn new(model_name: &str) -> Self {
    let groq_url: String = "https://api.groq.com/openai/v1/chat/completions".to_string();
    let api_key = std::env::var("GROQ_API_KEY").ok().context("Groq API key not found. Provide it or set GROQ_API_KEY env var.").unwrap();
    GroqProvider {groq_url, api_key, model_name: model_name.to_string(), client: Client::new()}
  }
}

#[async_trait]
impl LLMChatter for GroqProvider {
  async fn chat(&self, messages: Vec<ChatMessage>, config: &LLMModelConfig) -> Result<LLMResponse> {
    let request: GroqChatRequest = GroqChatRequest {
      model: self.model_name.clone(),
      messages: messages,
      temperature: config.temperature,
      max_completion_tokens: config.max_tokens,
      top_p: config.top_p,
      // stream: false,
      // stop: None,
    };

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", format!("Bearer {}", self.api_key).parse().unwrap());
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let response: Response = self.client.post(&self.groq_url).headers(headers).json(&request).send().await?; 

    if response.status().is_success() {
      let groq_response : GroqChatResponse = response.json().await?;
      // Pull out the first choice (or fail)
      let first : GroqChoice = groq_response.choices.into_iter().next().ok_or_else(|| anyhow!("No response choices received from Groq"))?;
      return Ok(LLMResponse{
        content: first.message.content
      });
    }
    else {
      log::error!("Error getting response from Groq: {:?}", response.status());
      return Ok(LLMResponse {content: "Error message for connecting to GROQ".to_string()});
    }


  }
}