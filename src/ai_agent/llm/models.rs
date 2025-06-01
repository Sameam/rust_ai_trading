use serde::{Serialize, Deserialize};
use std::env; // For environment variables
use std::sync::OnceLock;
use anyhow::{Result, anyhow};


use crate::ai_agent::llm::model_provider::{LLMModelConfig, ModelProvider, LLMChatter};
use crate::ai_agent::llm::groq::GroqProvider;

// --- LLMModelDescriptor (equivalent to Python's LLMModel class) ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMModel {
  pub display_name: String,
  pub model_name: String, // The actual name used in API calls
  pub provider: ModelProvider,
}

impl LLMModel {
  pub fn new(display_name: &str, model_name: &str, provider: ModelProvider) -> Self {
    LLMModel {
      display_name: display_name.to_string(),
      model_name: model_name.to_string(),
      provider,
    }
  }

    pub fn to_choice_tuple(&self) -> (String, String, String) {
      (
        self.display_name.clone(),
        self.model_name.clone(),
        self.provider.to_string(),
      )
    }

    pub fn has_json_mode(&self) -> bool {
      if self.is_deepseek() || self.is_gemini() {
        return false;
      }
      if self.is_ollama() {
        return self.model_name.contains("llama3") || self.model_name.contains("neural-chat");
      }
      true // Default for Anthropic, Groq, OpenAI (assuming they generally support it or their clients handle it)
    }

    pub fn is_deepseek(&self) -> bool {
        // Python used model_name.startswith("deepseek")
        // This might be too broad if provider is already DeepSeek.
        // Sticking to provider check for clarity, or combine if needed.
      self.provider == ModelProvider::DeepSeek // Or self.model_name.starts_with("deepseek")
    }

    pub fn is_gemini(&self) -> bool {
      self.provider == ModelProvider::Gemini // Or self.model_name.starts_with("gemini")
    }

    pub fn is_ollama(&self) -> bool {
      self.provider == ModelProvider::Ollama
    }
}

// --- Static Lists of Available Model Descriptors ---
fn available_models_data() -> Vec<LLMModel> {
  vec![
    // Anthropic - Using "latest" for haiku and sonnet as per Python.
    // Python had "claude-3.7-sonnet", using "claude-3-opus-20240229" as a more concrete example for a higher-end model.
    // Adjust these to the exact model identifiers you intend to use.
    LLMModel::new("[anthropic] claude-3.5-haiku", "claude-3-5-haiku-latest", ModelProvider::Anthropic),
    LLMModel::new("[anthropic] claude-3.5-sonnet", "claude-3-5-sonnet-latest", ModelProvider::Anthropic),
    LLMModel::new("[anthropic] claude-3-opus", "claude-3-opus-20240229", ModelProvider::Anthropic),

    // DeepSeek - Python names "deepseek-reasoner", "deepseek-chat".
    LLMModel::new("[deepseek] deepseek-coder", "deepseek-coder", ModelProvider::DeepSeek), // Example, check actual names
    LLMModel::new("[deepseek] deepseek-chat", "deepseek-chat", ModelProvider::DeepSeek),

    // Gemini - Python names "gemini-2.0-flash", "gemini-2.5-pro-exp-03-25". Using "latest" for simplicity.
    LLMModel::new("[gemini] gemini-1.5-flash", "gemini-1.5-flash-latest", ModelProvider::Gemini),
    LLMModel::new("[gemini] gemini-1.5-pro", "gemini-1.5-pro-latest", ModelProvider::Gemini),

    // Groq - Python names "meta-llama/llama-4-scout-17b-16e-instruct", "meta-llama/llama-4-maverick-17b-128e-instruct".
    // Groq typically lists models like "llama3-8b-8192". Using common Groq models.
    LLMModel::new("[groq] llama3-8b", "llama3-8b-8192", ModelProvider::Groq),
    LLMModel::new("[groq] llama3-70b", "llama3-70b-8192", ModelProvider::Groq),
    LLMModel::new("[groq] mixtral-8x7b", "mixtral-8x7b-32768", ModelProvider::Groq),

    // OpenAI - Python names "gpt-4.5-preview", "gpt-4o", "o3", "o4-mini".
    // "o3", "o4-mini" seem like custom aliases. Using standard OpenAI model names.
    LLMModel::new("[openai] gpt-3.5-turbo", "gpt-3.5-turbo", ModelProvider::OpenAI),
    LLMModel::new("[openai] gpt-4o", "gpt-4o", ModelProvider::OpenAI),
    LLMModel::new("[openai] gpt-4-turbo", "gpt-4-turbo", ModelProvider::OpenAI),
  ]
}

fn ollama_models_data() -> Vec<LLMModel> {
  vec![
    LLMModel::new("[google] gemma3 (4B)","gemma3:4b", ModelProvider::Ollama),
    LLMModel::new("[alibaba] qwen3 (4B)", "qwen3:4b", ModelProvider::Ollama),
    LLMModel::new("[meta] llama3.1 (8B)", "llama3.1:latest", ModelProvider::Ollama),
    LLMModel::new("[google] gemma3 (12B)", "gemma3:12b", ModelProvider::Ollama),
    LLMModel::new("[mistral] mistral-small3.1 (24B)", "mistral-small3.1", ModelProvider::Ollama),
    LLMModel::new("[google] gemma3 (27B)", "gemma3:27b", ModelProvider::Ollama),
    LLMModel::new("[alibaba] qwen3 (30B-a3B)", "qwen3:30b-a3b", ModelProvider::Ollama),
    LLMModel::new("[meta] llama-3.3 (70B)", "llama3.3:70b-instruct-q4_0", ModelProvider::Ollama),
  ]
}

pub static AVAILABLE_MODELS: OnceLock<Vec<LLMModel>> = OnceLock::new();
pub static OLLAMA_MODELS: OnceLock<Vec<LLMModel>> = OnceLock::new();


pub fn get_available_models() -> &'static [LLMModel] {
  AVAILABLE_MODELS.get_or_init(available_models_data).as_slice()
}

pub fn get_ollama_models() -> &'static [LLMModel] {
  OLLAMA_MODELS.get_or_init(ollama_models_data).as_slice()
}

pub fn get_llm_order() -> Vec<(String, String, String)> {
  get_available_models().iter().map(|m| m.to_choice_tuple()).collect()
}

pub fn get_ollama_llm_order() -> Vec<(String, String, String)> {
  get_ollama_models().iter().map(|m| m.to_choice_tuple()).collect()
}

pub fn get_model_info(model_name: &str) -> Option<&'static LLMModel> {
  get_available_models()
      .iter()
      .chain(get_ollama_models().iter())
      .find(|&model_desc| model_desc.model_name == model_name)
}

pub fn get_model(config: &LLMModelConfig) -> Result<Box<dyn LLMChatter>> {
  log::info!("Initializing LLM client for provider: {}, model: {}", config.provider,config.model_name);

  match config.provider {
    ModelProvider::Groq => {
      let client = GroqProvider::new(&config.model_name);
      return Ok(Box::new(client))
    }
    ModelProvider::OpenAI => {
      // let api_key = get_api_key_for_provider(&config.provider, &config.api_key)?;
      // Ok(Box::new(OpenAIProviderClient::new(api_key)))
      Err(anyhow!("OpenAI client not yet implemented"))
    }
    ModelProvider::Anthropic => {
      // let api_key = get_api_key_for_provider(&config.provider, &config.api_key)?;
      // Ok(Box::new(AnthropicProviderClient::new(api_key)))
      Err(anyhow!("Anthropic client not yet implemented"))
    }
    ModelProvider::DeepSeek => {
      // let api_key = get_api_key_for_provider(&config.provider, &config.api_key)?;
      // Ok(Box::new(DeepSeekProviderClient::new(api_key)))
      Err(anyhow!("DeepSeek client not yet implemented"))
    }
    ModelProvider::Gemini => {
      // let api_key = get_api_key_for_provider(&config.provider, &config.api_key)?;
      // Ok(Box::new(GeminiProviderClient::new(api_key)))
      Err(anyhow!("Gemini client not yet implemented"))
    }
    ModelProvider::Ollama => {
      let ollama_host = env::var("OLLAMA_HOST").unwrap_or_else(|_| "localhost".to_string());
      let default_base_url = format!("http://{}:11434", ollama_host);
      let base_url = config.base_url.as_ref().cloned().unwrap_or(default_base_url);
      // Ok(Box::new(OllamaProviderClient::new(base_url, config.model_name.clone())))
      log::info!("Ollama configured with base_url: {}", base_url);
      Err(anyhow!("Ollama client not yet implemented"))
    }
  }
}
