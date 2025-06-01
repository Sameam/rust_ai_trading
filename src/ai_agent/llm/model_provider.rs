use serde::{Serialize, Deserialize};
use std::str::FromStr;
use std::fmt;
use anyhow::{Result};
use async_trait::async_trait;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelProvider {
  Anthropic,
  DeepSeek,
  Gemini,
  Groq,
  OpenAI,
  Ollama,
}

impl ModelProvider {

  pub fn _as_str(&self) -> &'static str {
    match self {
      &ModelProvider::Anthropic => "Anthropic",
      &ModelProvider::DeepSeek => "DeepSeek",
      &ModelProvider::Gemini => "Gemini",
      &ModelProvider::Groq => "Groq",
      &ModelProvider::Ollama => "Ollama",
      &ModelProvider::OpenAI => "OpenAI"
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMModelConfig {
  pub provider: ModelProvider,
  pub model_name: String,
  pub api_key: Option<String>, // API keys are often better handled via env vars or a dedicated secret management
  pub base_url: Option<String>, // Useful for Ollama or other self-hosted/proxy setups
  pub temperature: Option<f32>,
  pub max_tokens: Option<u32>,
  pub top_p : Option<f32>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
  pub role: String, // e.g., "user", "assistant", "system"
  pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
  pub content: String,
  // You might include other details like token usage, finish reason, etc.
}

impl fmt::Display for ModelProvider {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ModelProvider::Anthropic => write!(f, "Anthropic"),
      ModelProvider::DeepSeek => write!(f, "DeepSeek"),
      ModelProvider::Gemini => write!(f, "Gemini"),
      ModelProvider::Groq => write!(f, "Groq"),
      ModelProvider::OpenAI => write!(f, "OpenAI"),
      ModelProvider::Ollama => write!(f, "Ollama"),
    }
  }
}

impl FromStr for ModelProvider {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.trim().to_lowercase().as_str() {
      "anthropic" => Ok(ModelProvider::Anthropic),  // Lowercase to match the lowercase input
      "deepseek" => Ok(ModelProvider::DeepSeek),    // Lowercase to match the lowercase input
      "gemini" => Ok(ModelProvider::Gemini),        // Lowercase to match the lowercase input
      "groq" => Ok(ModelProvider::Groq),            // Lowercase to match the lowercase input
      "ollama" => Ok(ModelProvider::Ollama),        // Lowercase to match the lowercase input
      "openai" => Ok(ModelProvider::OpenAI),        // Lowercase to match the lowercase input
      _ => Err(format!("Unknown model provider: {}", s)),
    }
  }
}

#[async_trait]
pub trait LLMChatter : Send + Sync {
  async fn chat(&self, messages: Vec<ChatMessage>,config : &LLMModelConfig) -> Result<LLMResponse>;
    
}




